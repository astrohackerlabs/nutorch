use crate::{
    Device, Kind, Tensor,
    operations::{Parameter, Params},
};

fn tch(op: &str, e: tch::TchError) -> crate::Error {
    ("torch_error", format!("{op}: {}", crate::tch_error(e)))
}

pub(super) fn build_module(
    kind: &str,
    args: &Params,
    guard: &std::sync::MutexGuard<'static, ()>,
) -> Result<super::kernels::NnModule, (&'static str, String)> {
    use super::kernels::NnModule;
    let int_arg = |name: &str| args.int(name);
    let tensor_arg = |name: &str| -> crate::Result<Option<Tensor>> {
        match args.values.get(name) {
            None => Ok(None),
            Some(Parameter::Tensor(t)) => t.alias_under_gate(guard).map(Some),
            _ => Err(crate::argument(format!(
                "nn {kind}: {name} must be a tensor"
            ))),
        }
    };
    let bool_arg = |name: &str| args.bool(name);
    match kind {
        "linear" => {
            let in_features = int_arg("in_features").ok_or((
                "bad_argument",
                "nn linear: usage: torch nn linear <in> <out> [--no-bias] [--weight T] [--bias-tensor T]".to_string(),
            ))?;
            let out_features = int_arg("out_features").ok_or((
                "bad_argument",
                "nn linear: missing out_features".to_string(),
            ))?;
            if in_features < 1 || out_features < 1 {
                return Err((
                    "bad_argument",
                    format!(
                        "nn linear: features must be >= 1, got {in_features} and {out_features}"
                    ),
                ));
            }
            let no_bias = bool_arg("no_bias");
            if no_bias && tensor_arg("bias_tensor")?.is_some() {
                return Err((
                    "bad_argument",
                    "nn linear: --bias-tensor contradicts --no-bias".to_string(),
                ));
            }
            // Explicit weights are DEEP-COPIED (state_dict-load semantics:
            // the caller's tensor is never aliased or mutated) and the
            // module's parameter gets requires_grad set LAST on the
            // post-copy tensor regardless of the source's setting.
            let weight = match tensor_arg("weight")? {
                Some(handle) => copy_module_param(&handle, &[out_features, in_features], "weight")?,
                None => init_linear_param(&[out_features, in_features], in_features)?,
            };
            let bias = if no_bias {
                None
            } else {
                Some(match tensor_arg("bias_tensor")? {
                    Some(handle) => copy_module_param(&handle, &[out_features], "bias")?,
                    None => init_linear_param(&[out_features], in_features)?,
                })
            };
            Ok(NnModule::Linear { weight, bias })
        }
        "conv1d" | "conv2d" | "conv_transpose2d" => {
            let req = |name: &str| -> Result<i64, (&'static str, String)> {
                int_arg(name).ok_or((
                    "bad_argument",
                    format!("nn {kind}: usage: torch nn {kind} <in_channels> <out_channels> <kernel_size> [flags]"),
                ))
            };
            let (in_ch, out_ch, k) = (
                req("in_channels")?,
                req("out_channels")?,
                req("kernel_size")?,
            );
            if in_ch < 1 || out_ch < 1 || k < 1 {
                return Err((
                    "bad_argument",
                    format!("nn {kind}: channels and kernel_size must be >= 1"),
                ));
            }
            let stride = int_arg("stride").unwrap_or(1);
            let padding = int_arg("padding").unwrap_or(0);
            let dilation = int_arg("dilation").unwrap_or(1);
            let groups = int_arg("groups").unwrap_or(1);
            // Validate BEFORE the division below: groups = 0 would panic
            // the connection thread (divide by zero), and non-divisible
            // channels would silently truncate the weight shape.
            if groups < 1 {
                return Err((
                    "bad_argument",
                    format!("nn {kind}: groups must be a positive integer, got {groups}"),
                ));
            }
            let (divided, label) = if kind == "conv_transpose2d" {
                (out_ch, "out_channels")
            } else {
                (in_ch, "in_channels")
            };
            if divided % groups != 0 {
                return Err((
                    "bad_argument",
                    format!(
                        "nn {kind}: {label} ({divided}) must be divisible by groups ({groups})"
                    ),
                ));
            }
            let no_bias = bool_arg("no_bias");
            if no_bias && tensor_arg("bias_tensor")?.is_some() {
                return Err((
                    "bad_argument",
                    format!("nn {kind}: --bias-tensor contradicts --no-bias"),
                ));
            }
            // Weight shapes: conv = [out, in/groups, k(,k)];
            // conv_transpose = [in, out/groups, k, k].
            let weight_shape: Vec<i64> = match kind {
                "conv1d" => vec![out_ch, in_ch / groups, k],
                "conv2d" => vec![out_ch, in_ch / groups, k, k],
                _ => vec![in_ch, out_ch / groups, k, k],
            };
            let fan_in = (in_ch / groups) * if kind == "conv1d" { k } else { k * k };
            let weight = match tensor_arg("weight")? {
                Some(handle) => copy_module_param(&handle, &weight_shape, "weight")?,
                None => init_linear_param(&weight_shape, fan_in)?,
            };
            let bias = if no_bias {
                None
            } else {
                Some(match tensor_arg("bias_tensor")? {
                    Some(handle) => copy_module_param(&handle, &[out_ch], "bias")?,
                    None => init_linear_param(&[out_ch], fan_in)?,
                })
            };
            Ok(match kind {
                "conv1d" => NnModule::Conv1d {
                    weight,
                    bias,
                    stride,
                    padding,
                    dilation,
                    groups,
                },
                "conv2d" => NnModule::Conv2d {
                    weight,
                    bias,
                    stride,
                    padding,
                    dilation,
                    groups,
                },
                _ => NnModule::ConvTranspose2d {
                    weight,
                    bias,
                    stride,
                    padding,
                    output_padding: int_arg("output_padding").unwrap_or(0),
                    groups,
                    dilation,
                },
            })
        }
        "embedding" => {
            let num = int_arg("num_embeddings").ok_or((
                "bad_argument",
                "nn embedding: usage: torch nn embedding <num_embeddings> <embedding_dim>"
                    .to_string(),
            ))?;
            let dim = int_arg("embedding_dim").ok_or((
                "bad_argument",
                "nn embedding: missing embedding_dim".to_string(),
            ))?;
            let weight = match tensor_arg("weight")? {
                Some(handle) => copy_module_param(&handle, &[num, dim], "weight")?,
                None => {
                    // PyTorch init: N(0, 1) — seeded CPU randn convention.
                    let t = Tensor::f_randn([num, dim], (Kind::Float, Device::Cpu))
                        .and_then(|t| t.f_to_device(Device::Mps))
                        .map_err(|e| tch("nn", e))?;
                    crate::mark_requires_grad(t)?
                }
            };
            Ok(NnModule::Embedding { weight })
        }
        "layer_norm" => {
            let shape = args.int_list("normalized_shape").ok_or_else(|| {
                crate::argument("nn layer_norm: normalized_shape must be an integer list")
            })?;
            let eps = float_module_arg(args, "eps", 1e-5);
            let weight = match tensor_arg("weight")? {
                Some(handle) => copy_module_param(&handle, &shape, "weight")?,
                None => ones_param(&shape)?,
            };
            let bias = match tensor_arg("bias_tensor")? {
                Some(handle) => copy_module_param(&handle, &shape, "bias")?,
                None => zeros_param(&shape)?,
            };
            Ok(NnModule::LayerNorm {
                shape,
                weight,
                bias,
                eps,
            })
        }
        "batch_norm" => {
            let features = int_arg("num_features").ok_or((
                "bad_argument",
                "nn batch_norm: usage: torch nn batch_norm <num_features>".to_string(),
            ))?;
            let eps = float_module_arg(args, "eps", 1e-5);
            let momentum = float_module_arg(args, "momentum", 0.1);
            let weight = ones_param(&[features])?;
            let bias = zeros_param(&[features])?;
            // Buffers, not parameters: no requires_grad.
            let running_mean = Tensor::f_zeros([features], (Kind::Float, Device::Mps))
                .map_err(|e| tch("nn", e))?;
            let running_var =
                Tensor::f_ones([features], (Kind::Float, Device::Mps)).map_err(|e| tch("nn", e))?;
            let num_batches_tracked =
                Tensor::f_zeros([], (Kind::Int64, Device::Mps)).map_err(|e| tch("nn", e))?;
            Ok(NnModule::BatchNorm {
                weight,
                bias,
                running_mean,
                running_var,
                num_batches_tracked,
                eps,
                momentum,
                training: true,
            })
        }
        "group_norm" => {
            let groups = int_arg("num_groups").ok_or((
                "bad_argument",
                "nn group_norm: usage: torch nn group_norm <num_groups> <num_channels>".to_string(),
            ))?;
            let channels = int_arg("num_channels").ok_or((
                "bad_argument",
                "nn group_norm: missing num_channels".to_string(),
            ))?;
            if groups < 1 || channels < 1 || channels % groups != 0 {
                return Err((
                    "bad_argument",
                    format!(
                        "nn group_norm: num_channels ({channels}) must be divisible by num_groups ({groups})"
                    ),
                ));
            }
            Ok(NnModule::GroupNorm {
                num_groups: groups,
                weight: ones_param(&[channels])?,
                bias: zeros_param(&[channels])?,
                eps: float_module_arg(args, "eps", 1e-5),
            })
        }
        "dropout" => {
            let p = float_module_arg(args, "p", 0.5);
            if !(0.0..=1.0).contains(&p) {
                return Err((
                    "bad_argument",
                    format!("nn dropout: p must be in [0, 1], got {p}"),
                ));
            }
            Ok(NnModule::Dropout { p, training: true })
        }
        "leaky_relu" => Ok(NnModule::LeakyRelu {
            slope: float_module_arg(args, "negative_slope", 0.01),
        }),
        "softmax" => {
            let dim = int_arg("dim").ok_or((
                "bad_argument",
                "nn softmax: usage: torch nn softmax <dim>".to_string(),
            ))?;
            Ok(NnModule::Softmax { dim })
        }
        "max_pool2d" | "avg_pool2d" => {
            let kernel = int_arg("kernel_size").ok_or((
                "bad_argument",
                format!("nn {kind}: usage: torch nn {kind} <kernel_size> [--stride S --padding P]"),
            ))?;
            let stride = int_arg("stride").unwrap_or(kernel);
            let padding = int_arg("padding").unwrap_or(0);
            Ok(if kind == "max_pool2d" {
                NnModule::MaxPool2d {
                    kernel,
                    stride,
                    padding,
                }
            } else {
                NnModule::AvgPool2d {
                    kernel,
                    stride,
                    padding,
                }
            })
        }
        "flatten" => Ok(NnModule::Flatten {
            // nn.Flatten defaults (1, -1) — NOT the table op's 0.
            start_dim: int_arg("start_dim").unwrap_or(1),
            end_dim: int_arg("end_dim").unwrap_or(-1),
        }),
        "relu" => Ok(NnModule::Relu),
        "sigmoid" => Ok(NnModule::Sigmoid),
        "tanh" => Ok(NnModule::Tanh),
        "gelu" => Ok(NnModule::Gelu),
        other => Err((
            "bad_argument",
            format!(
                "unknown module kind: {other} (expected linear, relu, sigmoid, tanh, gelu, or sequential)"
            ),
        )),
    }
}

/// Deep-copy an explicit weight into a module parameter: never aliases or
/// mutates the caller's tensor; requires_grad set LAST post-copy.
fn copy_module_param(
    source: &Tensor,
    expected_shape: &[i64],
    what: &str,
) -> Result<Tensor, (&'static str, String)> {
    let actual = source.size();
    if actual != expected_shape {
        return Err((
            "shape_mismatch",
            format!("nn: {what} must have shape {expected_shape:?}, got {actual:?}"),
        ));
    }
    let detached = source.f_detach().map_err(|e| tch("nn", e))?;
    let mut copy = detached.f_zeros_like().map_err(|e| tch("nn", e))?;
    copy.f_copy_(&detached).map_err(|e| tch("nn", e))?;
    crate::mark_requires_grad(copy)
}

fn ones_param(shape: &[i64]) -> Result<Tensor, (&'static str, String)> {
    let t = Tensor::f_ones(shape, (Kind::Float, Device::Mps)).map_err(|e| tch("nn", e))?;
    Ok(crate::mark_requires_grad(t)?)
}

fn zeros_param(shape: &[i64]) -> Result<Tensor, (&'static str, String)> {
    let t = Tensor::f_zeros(shape, (Kind::Float, Device::Mps)).map_err(|e| tch("nn", e))?;
    Ok(crate::mark_requires_grad(t)?)
}

fn float_module_arg(args: &Params, name: &str, default: f64) -> f64 {
    args.float(name).unwrap_or(default)
}

/// Construct an optimizer over a module's parameters (issue 0009 exp 4).
pub(super) fn build_optimizer(
    params: Vec<Tensor>,
    kind: &str,
    args: &Params,
) -> Result<super::kernels::Optimizer, (&'static str, String)> {
    use super::kernels::{OptimKind, Optimizer};
    let float_arg = |name: &str, default: f64| -> f64 { args.float(name).unwrap_or(default) };
    let bool_arg = |name: &str| -> bool { args.bool(name) };
    if params.is_empty() {
        return Err((
            "bad_argument",
            format!("nn {kind}: module has no parameters to optimize"),
        ));
    }
    let weight_decay = float_arg("weight_decay", if kind == "adamw" { 0.01 } else { 0.0 });
    let (optim_kind, default_lr) = match kind {
        "sgd" => {
            let momentum = float_arg("momentum", 0.0);
            let dampening = float_arg("dampening", 0.0);
            let nesterov = bool_arg("nesterov");
            if nesterov && (momentum <= 0.0 || dampening != 0.0) {
                return Err((
                    "bad_argument",
                    "nn sgd: nesterov requires momentum > 0 and dampening == 0".to_string(),
                ));
            }
            let lr = args
                .float("lr")
                .ok_or(("bad_argument", "nn sgd: --lr is required".to_string()))?;
            return Ok(Optimizer::new(
                OptimKind::Sgd {
                    momentum,
                    dampening,
                    nesterov,
                },
                lr,
                weight_decay,
                params,
            ));
        }
        "adam" | "adamw" => (
            OptimKind::Adam {
                beta1: float_arg("beta1", 0.9),
                beta2: float_arg("beta2", 0.999),
                eps: float_arg("eps", 1e-8),
                decoupled: kind == "adamw",
            },
            0.001,
        ),
        "rmsprop" => (
            OptimKind::RmsProp {
                alpha: float_arg("alpha", 0.99),
                eps: float_arg("eps", 1e-8),
                momentum: float_arg("momentum", 0.0),
            },
            0.01,
        ),
        other => return Err(("bad_argument", format!("unknown optimizer kind: {other}"))),
    };
    let lr = float_arg("lr", default_lr);
    Ok(Optimizer::new(optim_kind, lr, weight_decay, params))
}

/// PyTorch nn.Linear default init: U(-1/sqrt(in), 1/sqrt(in)) for both
/// weight and bias (kaiming_uniform(a=sqrt(5)) reduces to exactly this).
/// Drawn on the seeded CPU generator (the randn convention), moved to
/// MPS, requires_grad set LAST (the issue-0008 non-leaf trap).
fn init_linear_param(shape: &[i64], in_features: i64) -> Result<Tensor, (&'static str, String)> {
    let bound = 1.0 / (in_features as f64).sqrt();
    let uniform = Tensor::f_rand(shape, (Kind::Float, Device::Cpu))
        .and_then(|t| t.f_mul_scalar(2.0 * bound))
        .and_then(|t| t.f_sub_scalar(bound))
        .and_then(|t| t.f_to_device(Device::Mps))
        .map_err(|e| tch("nn", e))?;
    crate::mark_requires_grad(uniform)
}
