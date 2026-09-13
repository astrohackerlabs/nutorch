//! Typed libtorch operations for the native NuTorch shell.
//! No registry, serialization format, shell evaluator or transport lives here.
use std::sync::{Arc, Mutex, MutexGuard};
pub mod nn;
pub mod operations;
pub use tch::{Device, Kind, Tensor};

pub type Error = (&'static str, String);
pub type Result<T> = std::result::Result<T, Error>;

pub fn tch_error(e: tch::TchError) -> String {
    e.to_string()
        .lines()
        .next()
        .unwrap_or("internal torch error")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, time::Duration};

    fn x() -> SharedTensor {
        SharedTensor::create(
            &Data::List(vec![Data::Int(1), Data::Int(2), Data::Int(3)]),
            None,
            true,
        )
        .unwrap()
    }
    fn floats(values: &[f64]) -> Data {
        Data::List(values.iter().copied().map(Data::Float).collect())
    }

    #[test]
    fn aliases_gradients_snapshots_and_graph_lifetime_on_mps() {
        let x = x();
        assert_eq!(x.metadata().unwrap().device, "mps");
        let alias = x.clone();
        assert!(Arc::ptr_eq(&x.0, &alias.0));
        let intermediate = x.mul(&alias).unwrap();
        let weak_intermediate = Arc::downgrade(&intermediate.0);
        let loss = intermediate.sum(None, false).unwrap();
        drop(intermediate);
        assert!(weak_intermediate.upgrade().is_none());
        loss.backward().unwrap();
        let snapshot = alias.grad().unwrap();
        assert_eq!(snapshot.data().unwrap(), floats(&[2., 4., 6.]));
        x.mul(&x)
            .unwrap()
            .sum(None, false)
            .unwrap()
            .backward()
            .unwrap();
        assert_eq!(
            alias.grad().unwrap().data().unwrap(),
            floats(&[4., 8., 12.])
        );
        alias.zero_grad().unwrap();
        assert_eq!(x.grad().unwrap().data().unwrap(), floats(&[0., 0., 0.]));
        assert_eq!(snapshot.data().unwrap(), floats(&[2., 4., 6.]));
        let weak = Arc::downgrade(&x.0);
        drop(x);
        assert_eq!(alias.data().unwrap(), floats(&[1., 2., 3.]));
        drop(alias);
        assert!(
            weak.upgrade().is_none(),
            "the graph must not own Rust wrappers"
        );
        drop(loss);
    }

    #[test]
    fn detach_shares_storage_but_preserves_original_tracking() {
        let x = x();
        let detached = x.detach().unwrap();
        assert!(!detached.metadata().unwrap().requires_grad);
        assert!(x.metadata().unwrap().requires_grad);
        let _gate = gate().unwrap();
        let a = x.0.lock().unwrap();
        let mut b = detached.0.lock().unwrap();
        assert_eq!(a.data_ptr(), b.data_ptr());
        let _ = b.f_fill_(9.).unwrap();
        assert_eq!(
            to_data(&a.f_to_device(Device::Cpu).unwrap()).unwrap(),
            floats(&[9., 9., 9.])
        );
    }

    #[test]
    fn repeated_and_reversed_parallel_operands_do_not_deadlock() {
        let a = x();
        let b = x();
        let (tx, rx) = mpsc::channel();
        let mut threads = vec![];
        for reverse in [false, true] {
            let (a, b) = if reverse {
                (b.clone(), a.clone())
            } else {
                (a.clone(), b.clone())
            };
            let tx = tx.clone();
            threads.push(std::thread::spawn(move || {
                for _ in 0..20 {
                    a.mul(&a)
                        .unwrap()
                        .sum(None, false)
                        .unwrap()
                        .backward()
                        .unwrap();
                    a.zero_grad().unwrap();
                    assert_eq!(
                        a.add(&b, None).unwrap().data().unwrap(),
                        floats(&[2., 4., 6.])
                    );
                }
                tx.send(()).unwrap();
            }));
        }
        for _ in 0..2 {
            rx.recv_timeout(Duration::from_secs(20))
                .expect("tensor operations deadlocked");
        }
        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn validation_and_error_paths_leave_execution_usable() {
        let x = x();
        let error = x.backward().unwrap_err();
        assert_eq!(error.0, "bad_argument");
        assert!(error.1.contains("[3]"));
        let error = x.grad().unwrap_err();
        assert_eq!(error.0, "bad_argument");
        assert!(error.1.contains("run backward first"));
        let error = SharedTensor::create(&Data::Float(1.5), None, false)
            .unwrap()
            .backward()
            .unwrap_err();
        assert_eq!(error.0, "bad_argument");
        assert!(error.1.contains("does not require gradients"));
        assert!(SharedTensor::create(&Data::List(vec![]), None, false).is_err());
        assert!(SharedTensor::create(&Data::Int(1), Some("int64"), true).is_err());
        assert!(SharedTensor::create(&Data::Int(1), Some("nope"), false).is_err());
        assert_eq!(x.mul(&x).unwrap().data().unwrap(), floats(&[1., 4., 9.]));
        let poisoned = x.clone();
        let _ = std::thread::spawn(move || {
            let _lock = poisoned.0.lock().unwrap();
            panic!("test-only poison");
        })
        .join();
        assert!(x.data().unwrap_err().1.contains("poisoned"));
        assert!(
            SharedTensor::create(&Data::Int(2), None, false)
                .unwrap()
                .data()
                .is_ok()
        );
    }
}
fn torch(e: tch::TchError) -> Error {
    ("torch_error", tch_error(e))
}
fn argument(message: impl Into<String>) -> Error {
    ("bad_argument", message.into())
}

pub fn kind_name(kind: Kind) -> String {
    match kind {
        Kind::Float => "float32".into(),
        Kind::Double => "float64".into(),
        Kind::Half => "float16".into(),
        Kind::Int => "int32".into(),
        Kind::Int64 => "int64".into(),
        Kind::Int16 => "int16".into(),
        Kind::Int8 => "int8".into(),
        Kind::Uint8 => "uint8".into(),
        Kind::Bool => "bool".into(),
        other => format!("{other:?}").to_lowercase(),
    }
}
pub fn parse_kind(dtype: Option<&str>) -> std::result::Result<Kind, String> {
    match dtype {
        None | Some("float32" | "float") => Ok(Kind::Float),
        Some("float64" | "double") => Ok(Kind::Double),
        Some("int32" | "int") => Ok(Kind::Int),
        Some("int64" | "long") => Ok(Kind::Int64),
        Some("bool") => Ok(Kind::Bool),
        Some(s) => Err(format!(
            "invalid dtype: {s} (expected float32, float64, int32, int64, or bool)"
        )),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Data>),
}
impl Data {
    fn classify(&self, bools: &mut usize, numbers: &mut usize, nonfinite: &mut usize) {
        match self {
            Self::Bool(_) => *bools += 1,
            Self::Int(_) => *numbers += 1,
            Self::Float(f) => {
                *numbers += 1;
                if !f.is_finite() {
                    *nonfinite += 1;
                }
            }
            Self::List(xs) => {
                for x in xs {
                    x.classify(bools, numbers, nonfinite);
                }
            }
        }
    }
    fn float(&self) -> Result<f64> {
        match self {
            Self::Int(i) => Ok(*i as f64),
            Self::Float(f) => Ok(*f),
            Self::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Self::List(_) => Err(argument("ragged or mismatched nested list")),
        }
    }
}
pub fn resolve_kind(data: &Data, explicit: Option<Kind>) -> Result<Kind> {
    let (mut bools, mut numbers, mut nonfinite) = (0, 0, 0);
    data.classify(&mut bools, &mut numbers, &mut nonfinite);
    if let Some(k) = explicit {
        if matches!(
            k,
            Kind::Int | Kind::Int64 | Kind::Int8 | Kind::Int16 | Kind::Uint8
        ) && nonfinite > 0
        {
            return Err(argument(
                "non-finite values cannot be cast to an integer dtype",
            ));
        }
        return Ok(k);
    }
    if bools > 0 && numbers > 0 {
        return Err(argument(
            "mixed booleans and numbers (pass an explicit --dtype to cast)",
        ));
    }
    Ok(if bools > 0 { Kind::Bool } else { Kind::Float })
}
pub fn from_data(data: &Data, kind: Kind, device: Device) -> Result<Tensor> {
    let tensor = match data {
        Data::List(items) => {
            if items.is_empty() {
                return Err(argument("list cannot be empty"));
            }
            if matches!(items[0], Data::List(_)) {
                let tensors = items
                    .iter()
                    .map(|v| from_data(v, kind, device))
                    .collect::<Result<Vec<_>>>()?;
                return Tensor::f_stack(&tensors, 0).map_err(|e| {
                    argument(format!(
                        "ragged or mismatched nested list: {}",
                        tch_error(e)
                    ))
                });
            }
            if items.iter().all(|v| matches!(v, Data::Int(_))) {
                let values: Vec<i64> = items
                    .iter()
                    .map(|v| match v {
                        Data::Int(i) => *i,
                        _ => unreachable!(),
                    })
                    .collect();
                Tensor::f_from_slice(&values).map_err(torch)?
            } else {
                let values = items.iter().map(Data::float).collect::<Result<Vec<_>>>()?;
                Tensor::f_from_slice(&values).map_err(torch)?
            }
        }
        Data::Int(i) => Tensor::f_from_slice(&[*i])
            .and_then(|t| t.f_reshape([]))
            .map_err(torch)?,
        scalar => Tensor::f_from_slice(&[scalar.float()?])
            .and_then(|t| t.f_reshape([]))
            .map_err(torch)?,
    };
    tensor
        .f_to_kind(kind)
        .and_then(|t| t.f_to_device(device))
        .map_err(torch)
}
pub fn to_data(tensor: &Tensor) -> Result<Data> {
    let dims = tensor.size();
    if dims.is_empty() {
        return match tensor.kind() {
            Kind::Int | Kind::Int8 | Kind::Int16 | Kind::Int64 | Kind::Uint8 => {
                tensor.f_int64_value(&[]).map(Data::Int).map_err(torch)
            }
            Kind::Float | Kind::Double | Kind::Half => {
                tensor.f_double_value(&[]).map(Data::Float).map_err(torch)
            }
            Kind::Bool => tensor
                .f_int64_value(&[])
                .map(|i| Data::Bool(i != 0))
                .map_err(torch),
            k => Err(argument(format!("cannot convert tensor of type {k:?}"))),
        };
    }
    (0..dims[0])
        .map(|i| to_data(&tensor.f_get(i).map_err(torch)?))
        .collect::<Result<Vec<_>>>()
        .map(Data::List)
}
pub fn mark_requires_grad(tensor: Tensor) -> Result<Tensor> {
    if !matches!(tensor.kind(), Kind::Float | Kind::Double | Kind::Half) {
        return Err((
            "bad_dtype",
            format!(
                "only floating point tensors can require gradients, got {}",
                kind_name(tensor.kind())
            ),
        ));
    }
    tensor.f_set_requires_grad(true).map_err(torch)
}

#[derive(Clone, Copy)]
pub enum Scalar {
    Int(i64),
    Float(f64),
}
pub fn add(a: &Tensor, b: &Tensor, alpha: Option<Scalar>) -> Result<Tensor> {
    match alpha {
        None | Some(Scalar::Int(1)) => a.f_add(b).map_err(torch),
        Some(alpha) => {
            let scaled = match alpha {
                Scalar::Int(i) => b.f_mul_scalar(i),
                Scalar::Float(f) => b.f_mul_scalar(f),
            }
            .map_err(torch)?;
            a.f_add(&scaled).map_err(torch)
        }
    }
}
pub fn mul(a: &Tensor, b: &Tensor) -> Result<Tensor> {
    a.f_mul(b).map_err(torch)
}
pub fn sum(a: &Tensor, dim: Option<i64>, keepdim: bool) -> Result<Tensor> {
    match dim {
        Some(d) => a.f_sum_dim_intlist(Some(&[d][..]), keepdim, None::<Kind>),
        None => a.f_sum(None::<Kind>),
    }
    .map_err(torch)
}
pub fn backward(a: &Tensor) -> Result<()> {
    if !a.requires_grad() {
        return Err(argument(
            "backward: tensor does not require gradients (create with --requires_grad)",
        ));
    }
    if a.numel() != 1 {
        return Err(argument(format!(
            "backward: needs a scalar loss, got shape {:?} ({} elements) — reduce first (e.g. sum or mean)",
            a.size(),
            a.numel()
        )));
    }
    a.f_backward().map_err(torch)
}
pub fn grad(a: &Tensor) -> Result<Tensor> {
    let grad = a.f_grad().map_err(torch)?;
    if !grad.defined() {
        return Err(argument("no gradient: run backward first"));
    }
    let detached = grad.f_detach().map_err(torch)?;
    let mut snapshot = detached.f_zeros_like().map_err(torch)?;
    snapshot.f_copy_(&detached).map_err(torch)?;
    Ok(snapshot)
}
pub fn detach(a: &Tensor) -> Result<Tensor> {
    a.f_detach().map_err(torch)
}
pub fn zero_grad(a: &Tensor) -> Result<()> {
    let mut grad = a.f_grad().map_err(torch)?;
    if grad.defined() {
        let _ = grad.f_detach_().map_err(torch)?;
        let _ = grad.f_zero_().map_err(torch)?;
    }
    Ok(())
}

// Backward can mutate leaves not present in its explicit operand list. A global
// execution gate therefore serializes native operations, including display and
// detach views, before locking any wrappers. It owns no tensors. Parallel Nu
// tasks are safe but tensor kernels are intentionally submitted serially.
static EXECUTION: Mutex<()> = Mutex::new(());
fn gate() -> Result<MutexGuard<'static, ()>> {
    EXECUTION
        .lock()
        .map_err(|_| argument("tensor execution lock poisoned"))
}
#[derive(Clone)]
pub struct SharedTensor(Arc<Mutex<Tensor>>);
impl std::fmt::Debug for SharedTensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SharedTensor")
    }
}
#[derive(Debug)]
pub struct Metadata {
    pub shape: Vec<i64>,
    pub dtype: String,
    pub device: String,
    pub requires_grad: bool,
}
impl SharedTensor {
    // Caller must retain the execution gate while using the returned alias.
    // Shallow cloning preserves TensorImpl/storage/autograd identity, without
    // retaining wrapper locks when the same tensor occurs in several inputs.
    fn alias_under_gate(&self, _gate: &MutexGuard<'static, ()>) -> Result<Tensor> {
        Ok(self
            .0
            .lock()
            .map_err(|_| argument("tensor lock poisoned"))?
            .shallow_clone())
    }
    fn new(tensor: Tensor) -> Self {
        Self(Arc::new(Mutex::new(tensor)))
    }
    pub fn create(data: &Data, dtype: Option<&str>, requires_grad: bool) -> Result<Self> {
        let _gate = gate()?;
        if !tch::utils::has_mps() {
            return Err(argument(
                "NuTorch requires an Apple-silicon Mac with MPS (GPU-only by design)",
            ));
        }
        let explicit = dtype
            .map(|s| parse_kind(Some(s)))
            .transpose()
            .map_err(|e| ("bad_dtype", e))?;
        let tensor = from_data(data, resolve_kind(data, explicit)?, Device::Mps)?;
        Ok(Self::new(if requires_grad {
            mark_requires_grad(tensor)?
        } else {
            tensor
        }))
    }
    fn with<R>(&self, f: impl FnOnce(&Tensor) -> Result<R>) -> Result<R> {
        let _gate = gate()?;
        let a = self
            .0
            .lock()
            .map_err(|_| argument("tensor lock poisoned"))?;
        f(&a)
    }
    fn binary(
        &self,
        other: &Self,
        f: impl FnOnce(&Tensor, &Tensor) -> Result<Tensor>,
    ) -> Result<Self> {
        let _gate = gate()?;
        let a = self
            .0
            .lock()
            .map_err(|_| argument("tensor lock poisoned"))?;
        let tensor = if Arc::ptr_eq(&self.0, &other.0) {
            f(&a, &a)?
        } else {
            let b = other
                .0
                .lock()
                .map_err(|_| argument("tensor lock poisoned"))?;
            f(&a, &b)?
        };
        Ok(Self::new(tensor))
    }
    pub fn add(&self, other: &Self, alpha: Option<Scalar>) -> Result<Self> {
        self.binary(other, |a, b| add(a, b, alpha))
    }
    pub fn mul(&self, other: &Self) -> Result<Self> {
        self.binary(other, mul)
    }
    pub fn sum(&self, dim: Option<i64>, keepdim: bool) -> Result<Self> {
        self.with(|a| sum(a, dim, keepdim).map(Self::new))
    }
    pub fn backward(&self) -> Result<()> {
        self.with(backward)
    }
    pub fn grad(&self) -> Result<Self> {
        self.with(|a| grad(a).map(Self::new))
    }
    pub fn detach(&self) -> Result<Self> {
        self.with(|a| detach(a).map(Self::new))
    }
    pub fn zero_grad(&self) -> Result<()> {
        self.with(zero_grad)
    }
    pub fn data(&self) -> Result<Data> {
        self.with(|a| to_data(&a.f_to_device(Device::Cpu).map_err(torch)?))
    }
    pub fn metadata(&self) -> Result<Metadata> {
        self.with(|a| {
            Ok(Metadata {
                shape: a.size(),
                dtype: kind_name(a.kind()),
                device: format!("{:?}", a.device()).to_lowercase(),
                requires_grad: a.requires_grad(),
            })
        })
    }
}
