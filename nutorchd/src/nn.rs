//! Neural-network modules (issue 0009): daemon-resident objects composed
//! from the shell. Own parameter management, no tch VarStore — the
//! decision record lives in issues/0009-nn-optim/02-module-foundation.md
//! (composition over handles needs parameter identity under our control;
//! optimizers will hold shallow-clone references to these tensors).

use tch::Tensor;

use crate::convert::tch_error;

pub enum NnModule {
    Linear {
        weight: Tensor,
        bias: Option<Tensor>,
    },
    Relu,
    Sigmoid,
    Tanh,
    Gelu,
    Sequential {
        children: Vec<NnModule>,
    },
}

impl NnModule {
    pub fn kind_name(&self) -> &'static str {
        match self {
            NnModule::Linear { .. } => "linear",
            NnModule::Relu => "relu",
            NnModule::Sigmoid => "sigmoid",
            NnModule::Tanh => "tanh",
            NnModule::Gelu => "gelu",
            NnModule::Sequential { .. } => "sequential",
        }
    }

    pub fn forward(&self, input: &Tensor) -> Result<Tensor, String> {
        match self {
            NnModule::Linear { weight, bias } => input
                .f_linear(weight, bias.as_ref())
                .map_err(|e| format!("linear forward: {}", tch_error(e))),
            NnModule::Relu => input.f_relu().map_err(tch_error),
            NnModule::Sigmoid => input.f_sigmoid().map_err(tch_error),
            NnModule::Tanh => input.f_tanh().map_err(tch_error),
            NnModule::Gelu => input.f_gelu("none").map_err(tch_error),
            NnModule::Sequential { children } => {
                let mut current = input.shallow_clone();
                for child in children {
                    current = child.forward(&current)?;
                }
                Ok(current)
            }
        }
    }

    /// Depth-first, weight before bias (PyTorch's .parameters() order).
    pub fn parameters(&self) -> Vec<&Tensor> {
        let mut params = Vec::new();
        self.collect_parameters(&mut params);
        params
    }

    fn collect_parameters<'a>(&'a self, params: &mut Vec<&'a Tensor>) {
        match self {
            NnModule::Linear { weight, bias } => {
                params.push(weight);
                if let Some(bias) = bias {
                    params.push(bias);
                }
            }
            NnModule::Sequential { children } => {
                for child in children {
                    child.collect_parameters(params);
                }
            }
            _ => {}
        }
    }

    pub fn param_bytes(&self) -> u64 {
        self.parameters()
            .iter()
            .map(|t| t.numel() as u64 * t.kind().elt_size_in_bytes() as u64)
            .sum()
    }

    /// One line per fact, for `torch nn info`.
    pub fn describe(&self) -> Vec<String> {
        let params = self.parameters();
        let elements: i64 = params.iter().map(|t| t.numel() as i64).sum();
        let mut lines = vec![
            format!("kind: {}", self.kind_name()),
            format!(
                "parameters: {} tensor(s), {} element(s)",
                params.len(),
                elements
            ),
        ];
        match self {
            NnModule::Linear { weight, bias } => {
                let shape = weight.size();
                lines.push(format!(
                    "features: {} in, {} out, bias: {}",
                    shape[1],
                    shape[0],
                    bias.is_some()
                ));
            }
            NnModule::Sequential { children } => {
                let kinds: Vec<&str> = children.iter().map(|c| c.kind_name()).collect();
                lines.push(format!("children: {}", kinds.join(" -> ")));
            }
            _ => {}
        }
        lines
    }
}
