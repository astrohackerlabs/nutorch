//! Shared native modules and optimizers. All kernel calls, state inspection and
//! mutation use the same execution gate as ordinary tensors and autograd.
mod build;
mod kernels;
pub mod schema;

use crate::{Result, SharedTensor, Tensor, argument, gate, operations::Params, torch};
use kernels::{NnModule, Optimizer};
use std::{
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

enum Module {
    Leaf(NnModule),
    Sequential(Vec<SharedModule>),
}
#[derive(Clone)]
pub struct SharedModule(Arc<Mutex<Module>>);
#[derive(Clone)]
pub struct SharedOptimizer(Arc<Mutex<Optimizer>>);
impl std::fmt::Debug for SharedModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SharedModule")
    }
}
impl std::fmt::Debug for SharedOptimizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SharedOptimizer")
    }
}
type Guard = MutexGuard<'static, ()>;
impl SharedModule {
    pub fn create(kind: &str, params: &Params) -> Result<Self> {
        schema::validate(kind, params)?;
        let guard = gate()?;
        if !tch::utils::has_mps() {
            return Err(argument("NuTorch requires an Apple-silicon Mac with MPS"));
        }
        Ok(Self(Arc::new(Mutex::new(Module::Leaf(
            build::build_module(kind, params, &guard)?,
        )))))
    }
    /// Children are immutable after construction, so cycles cannot be formed.
    /// Reject duplicate nodes anywhere in the composition, including overlapping
    /// subtrees, before creating a parent. Existing child aliases remain usable.
    pub fn sequential(children: Vec<Self>) -> Result<Self> {
        let guard = gate()?;
        if children.is_empty() {
            return Err(argument("nn sequential: needs at least one child module"));
        }
        let mut seen = std::collections::HashSet::new();
        for child in &children {
            child.collect_nodes(&guard, &mut seen)?;
        }
        Ok(Self(Arc::new(Mutex::new(Module::Sequential(children)))))
    }
    fn lock(&self, _guard: &Guard) -> Result<MutexGuard<'_, Module>> {
        self.0.lock().map_err(|_| argument("module lock poisoned"))
    }
    fn collect_nodes(
        &self,
        guard: &Guard,
        seen: &mut std::collections::HashSet<usize>,
    ) -> Result<()> {
        if !seen.insert(Arc::as_ptr(&self.0) as usize) {
            return Err(argument("nn sequential: duplicate child module"));
        }
        if let Module::Sequential(children) = &*self.lock(guard)? {
            for child in children {
                child.collect_nodes(guard, seen)?;
            }
        }
        Ok(())
    }
    fn forward_under_gate(&self, input: &Tensor, guard: &Guard) -> Result<Tensor> {
        match &*self.lock(guard)? {
            Module::Leaf(module) => module.forward(input).map_err(|e| ("torch_error", e)),
            Module::Sequential(children) => {
                let mut result = input.shallow_clone();
                for child in children {
                    result = child.forward_under_gate(&result, guard)?;
                }
                Ok(result)
            }
        }
    }
    pub fn forward(&self, input: &SharedTensor) -> Result<SharedTensor> {
        let guard = gate()?;
        let tensor = input.alias_under_gate(&guard)?;
        self.forward_under_gate(&tensor, &guard)
            .map(SharedTensor::new)
    }
    fn parameters_under_gate(&self, guard: &Guard) -> Result<Vec<Tensor>> {
        match &*self.lock(guard)? {
            Module::Leaf(module) => Ok(module
                .parameters()
                .into_iter()
                .map(Tensor::shallow_clone)
                .collect()),
            Module::Sequential(children) => {
                let mut params = vec![];
                for child in children {
                    params.extend(child.parameters_under_gate(guard)?);
                }
                Ok(params)
            }
        }
    }
    pub fn parameters(&self) -> Result<Vec<SharedTensor>> {
        let guard = gate()?;
        Ok(self
            .parameters_under_gate(&guard)?
            .into_iter()
            .map(SharedTensor::new)
            .collect())
    }
    fn set_training_under_gate(&self, training: bool, guard: &Guard) -> Result<()> {
        match &mut *self.lock(guard)? {
            Module::Leaf(module) => module.set_training(training),
            Module::Sequential(children) => {
                for child in children {
                    child.set_training_under_gate(training, guard)?;
                }
            }
        }
        Ok(())
    }
    pub fn set_training(&self, training: bool) -> Result<()> {
        self.set_training_under_gate(training, &gate()?)
    }
    fn training_under_gate(&self, guard: &Guard) -> Result<bool> {
        match &*self.lock(guard)? {
            Module::Leaf(module) => Ok(module.is_training()),
            Module::Sequential(children) => {
                let mut training = false;
                for child in children {
                    training |= child.training_under_gate(guard)?;
                }
                Ok(training)
            }
        }
    }
    pub fn is_training(&self) -> Result<bool> {
        self.training_under_gate(&gate()?)
    }
    pub fn zero_grad(&self) -> Result<()> {
        let guard = gate()?;
        kernels::zero_grads(&self.parameters_under_gate(&guard)?).map_err(|e| ("torch_error", e))
    }
    fn kind_under_gate(&self, guard: &Guard) -> Result<&'static str> {
        Ok(match &*self.lock(guard)? {
            Module::Leaf(module) => module.kind_name(),
            Module::Sequential(_) => "sequential",
        })
    }
    pub fn describe(&self) -> Result<Vec<String>> {
        let guard = gate()?;
        match &*self.lock(&guard)? {
            Module::Leaf(module) => Ok(module.describe()),
            Module::Sequential(children) => {
                let mut params = vec![];
                let mut kinds = vec![];
                for child in children {
                    params.extend(child.parameters_under_gate(&guard)?);
                    kinds.push(child.kind_under_gate(&guard)?);
                }
                let mut training = false;
                for child in children {
                    training |= child.training_under_gate(&guard)?;
                }
                Ok(vec![
                    "kind: sequential".into(),
                    format!(
                        "parameters: {} tensor(s), {} element(s)",
                        params.len(),
                        params.iter().map(Tensor::numel).sum::<usize>()
                    ),
                    format!("children: {}", kinds.join(" -> ")),
                    format!("training: {training}"),
                ])
            }
        }
    }
    fn state_under_gate(&self, guard: &Guard) -> Result<Vec<(String, Tensor)>> {
        match &*self.lock(guard)? {
            Module::Leaf(module) => Ok(module
                .named_state()
                .into_iter()
                .map(|(name, tensor)| (name, tensor.shallow_clone()))
                .collect()),
            Module::Sequential(children) => {
                let mut state = vec![];
                for (index, child) in children.iter().enumerate() {
                    state.extend(
                        child
                            .state_under_gate(guard)?
                            .into_iter()
                            .map(|(name, tensor)| (format!("{index}.{name}"), tensor)),
                    );
                }
                Ok(state)
            }
        }
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        let guard = gate()?;
        let state = self
            .state_under_gate(&guard)?
            .into_iter()
            .map(|(name, tensor)| {
                tensor
                    .f_detach()
                    .and_then(|t| t.f_to_device(crate::Device::Cpu))
                    .map(|t| (name, t))
                    .map_err(torch)
            })
            .collect::<Result<Vec<_>>>()?;
        Tensor::write_safetensors(&state, path)
            .map_err(|e| argument(format!("save: {}", crate::tch_error(e))))
    }
    pub fn load(&self, path: &Path) -> Result<()> {
        let guard = gate()?;
        if !path.exists() {
            return Err(argument(format!("load: no such file: {}", path.display())));
        }
        let file = Tensor::read_safetensors(path).map_err(torch)?;
        let targets = self.state_under_gate(&guard)?;
        for (name, _) in &targets {
            if !file.iter().any(|(key, _)| key == name) {
                return Err(argument(format!("load: missing key in file: {name}")));
            }
        }
        for (name, tensor) in &file {
            let Some((_, target)) = targets.iter().find(|(key, _)| key == name) else {
                return Err(argument(format!("load: unexpected key in file: {name}")));
            };
            if tensor.size() != target.size() {
                return Err(argument(format!(
                    "load: shape mismatch for {name}: module {:?}, file {:?}",
                    target.size(),
                    tensor.size()
                )));
            }
        }
        // Prepare every device transfer and dtype conversion before the first
        // mutation. Retain snapshots to roll back a failed device copy.
        tch::no_grad(|| {
            let mut prepared = vec![];
            for (name, target) in &targets {
                let source = &file
                    .iter()
                    .find(|(key, _)| key == name)
                    .expect("validated key")
                    .1;
                // Preserve baseline dtype compatibility: transfer first, so
                // unsupported MPS dtypes (notably float64) remain errors.
                // Preparing all entries still makes rejection atomic.
                let converted = source
                    .f_to_device(target.device())
                    .and_then(|t| t.f_to_kind(target.kind()))
                    .map_err(torch)?;
                let mut snapshot = target.f_zeros_like().map_err(torch)?;
                snapshot.f_copy_(target).map_err(torch)?;
                prepared.push((target.shallow_clone(), converted, snapshot));
            }
            for index in 0..prepared.len() {
                let (target, source, _) = &mut prepared[index];
                if let Err(error) = target.f_copy_(source) {
                    for (target, _, snapshot) in &mut prepared[..=index] {
                        target.f_copy_(snapshot).map_err(torch)?;
                    }
                    return Err(torch(error));
                }
            }
            Ok(())
        })
    }
}

impl SharedOptimizer {
    pub fn create(kind: &str, module: &SharedModule, params: &Params) -> Result<Self> {
        schema::validate(kind, params)?;
        let guard = gate()?;
        let tensors = module.parameters_under_gate(&guard)?;
        Ok(Self(Arc::new(Mutex::new(build::build_optimizer(
            tensors, kind, params,
        )?))))
    }
    fn lock(&self, _guard: &Guard) -> Result<MutexGuard<'_, Optimizer>> {
        self.0
            .lock()
            .map_err(|_| argument("optimizer lock poisoned"))
    }
    pub fn step(&self) -> Result<()> {
        let guard = gate()?;
        tch::no_grad(|| self.lock(&guard)?.step().map_err(|e| ("torch_error", e)))
    }
    pub fn zero_grad(&self) -> Result<()> {
        self.lock(&gate()?)?
            .zero_grad()
            .map_err(|e| ("torch_error", e))
    }
    pub fn set_lr(&self, lr: f64) -> Result<()> {
        if !lr.is_finite() || lr < 0.0 {
            return Err(argument("learning rate must be finite and nonnegative"));
        }
        self.lock(&gate()?)?.lr = lr;
        Ok(())
    }
    pub fn describe(&self) -> Result<Vec<String>> {
        let guard = gate()?;
        let opt = self.lock(&guard)?;
        Ok(vec![
            format!("kind: {}", opt.kind_name()),
            format!("parameters: {}", opt.param_count()),
            format!("lr: {}", opt.lr),
            format!("weight_decay: {}", opt.weight_decay),
            format!("state_bytes: {}", opt.state_bytes()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Data, operations::Parameter};
    fn model() -> SharedModule {
        SharedModule::create(
            "linear",
            &Params {
                values: [
                    ("in_features".into(), Parameter::Int(1)),
                    ("out_features".into(), Parameter::Int(1)),
                ]
                .into(),
            },
        )
        .unwrap()
    }
    #[test]
    fn wrapper_lifetimes_have_no_global_roots() {
        for _ in 0..20 {
            let module = model();
            let weak_module = Arc::downgrade(&module.0);
            let parameter = module.parameters().unwrap().remove(0);
            let weak_parameter = Arc::downgrade(&parameter.0);
            let optimizer = SharedOptimizer::create("adam", &module, &Params::default()).unwrap();
            let weak_optimizer = Arc::downgrade(&optimizer.0);
            let composed = SharedModule::sequential(vec![module.clone()]).unwrap();
            drop(module);
            assert!(weak_module.upgrade().is_some());
            drop(composed);
            assert!(weak_module.upgrade().is_none());
            parameter
                .mul(&parameter)
                .unwrap()
                .sum(None, false)
                .unwrap()
                .backward()
                .unwrap();
            optimizer.step().unwrap();
            drop(optimizer);
            assert!(weak_optimizer.upgrade().is_none());
            assert!(parameter.data().is_ok());
            drop(parameter);
            assert!(weak_parameter.upgrade().is_none());
        }
    }
    #[test]
    fn module_optimizer_and_parameter_views_share_the_execution_gate() {
        let module = model();
        let x = SharedTensor::create(
            &Data::List(vec![Data::List(vec![Data::Float(1.)])]),
            None,
            false,
        )
        .unwrap();
        let optimizer = SharedOptimizer::create("adam", &module, &Params::default()).unwrap();
        let parameter = module.parameters().unwrap().remove(0);
        module
            .forward(&x)
            .unwrap()
            .sum(None, false)
            .unwrap()
            .backward()
            .unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let second_tx = tx.clone();
        let first = std::thread::spawn(move || {
            for _ in 0..30 {
                optimizer.step().unwrap();
            }
            tx.send(()).unwrap();
        });
        let second = std::thread::spawn(move || {
            for _ in 0..30 {
                module.forward(&x).unwrap();
                parameter.data().unwrap();
            }
            second_tx.send(()).unwrap();
        });
        for _ in 0..2 {
            rx.recv_timeout(std::time::Duration::from_secs(20))
                .expect("native optimizer/module access deadlocked");
        }
        first.join().unwrap();
        second.join().unwrap();
    }
}
