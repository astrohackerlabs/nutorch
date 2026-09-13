//! Constructor arguments shared by core validation and native shell signatures.
use crate::{
    Result, argument,
    operations::{Parameter, Params},
};

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Int,
    Number,
    IntList,
    Bool,
    Tensor,
}
#[derive(Clone, Copy)]
pub struct Argument {
    pub name: &'static str,
    pub kind: Kind,
    pub positional: bool,
}
pub const MODULES: &[&str] = &[
    "linear",
    "relu",
    "sigmoid",
    "tanh",
    "gelu",
    "sequential",
    "conv1d",
    "conv2d",
    "conv_transpose2d",
    "embedding",
    "layer_norm",
    "batch_norm",
    "group_norm",
    "dropout",
    "leaky_relu",
    "softmax",
    "max_pool2d",
    "avg_pool2d",
    "flatten",
];
pub const OPTIMIZERS: &[&str] = &["sgd", "adam", "adamw", "rmsprop"];
pub fn arguments(kind: &str) -> Result<Vec<Argument>> {
    use Kind::*;
    let positionals: &[(&str, Kind)] = match kind {
        "linear" => &[("in_features", Int), ("out_features", Int)],
        "conv1d" | "conv2d" | "conv_transpose2d" => &[
            ("in_channels", Int),
            ("out_channels", Int),
            ("kernel_size", Int),
        ],
        "embedding" => &[("num_embeddings", Int), ("embedding_dim", Int)],
        "layer_norm" => &[("normalized_shape", IntList)],
        "batch_norm" => &[("num_features", Int)],
        "group_norm" => &[("num_groups", Int), ("num_channels", Int)],
        "softmax" => &[("dim", Int)],
        "max_pool2d" | "avg_pool2d" => &[("kernel_size", Int)],
        k if MODULES.contains(&k) || OPTIMIZERS.contains(&k) => &[],
        _ => {
            return Err(argument(format!(
                "unknown module or optimizer kind: {kind}"
            )));
        }
    };
    let flags: &[(&str, Kind)] = match kind {
        "linear" => &[
            ("weight", Tensor),
            ("bias_tensor", Tensor),
            ("no_bias", Bool),
        ],
        "conv1d" | "conv2d" => &[
            ("weight", Tensor),
            ("bias_tensor", Tensor),
            ("no_bias", Bool),
            ("stride", Int),
            ("padding", Int),
            ("dilation", Int),
            ("groups", Int),
        ],
        "conv_transpose2d" => &[
            ("weight", Tensor),
            ("bias_tensor", Tensor),
            ("no_bias", Bool),
            ("stride", Int),
            ("padding", Int),
            ("dilation", Int),
            ("groups", Int),
            ("output_padding", Int),
        ],
        "embedding" => &[("weight", Tensor)],
        "layer_norm" => &[("weight", Tensor), ("bias_tensor", Tensor), ("eps", Number)],
        "batch_norm" => &[("eps", Number), ("momentum", Number)],
        "group_norm" => &[("eps", Number)],
        "dropout" => &[("p", Number)],
        "leaky_relu" => &[("negative_slope", Number)],
        "max_pool2d" | "avg_pool2d" => &[("stride", Int), ("padding", Int)],
        "flatten" => &[("start_dim", Int), ("end_dim", Int)],
        "sgd" => &[
            ("lr", Number),
            ("weight_decay", Number),
            ("momentum", Number),
            ("dampening", Number),
            ("nesterov", Bool),
        ],
        "adam" | "adamw" => &[
            ("lr", Number),
            ("weight_decay", Number),
            ("beta1", Number),
            ("beta2", Number),
            ("eps", Number),
        ],
        "rmsprop" => &[
            ("lr", Number),
            ("weight_decay", Number),
            ("alpha", Number),
            ("eps", Number),
            ("momentum", Number),
        ],
        _ => &[],
    };
    Ok(positionals
        .iter()
        .map(|&(name, kind)| Argument {
            name,
            kind,
            positional: true,
        })
        .chain(flags.iter().map(|&(name, kind)| Argument {
            name,
            kind,
            positional: false,
        }))
        .collect())
}
pub(super) fn validate(kind: &str, params: &Params) -> Result<()> {
    let args = arguments(kind)?;
    for name in params.values.keys() {
        if !args.iter().any(|a| a.name == name) {
            return Err(argument(format!("nn {kind}: unknown parameter: {name}")));
        }
    }
    for arg in args {
        let Some(value) = params.values.get(arg.name) else {
            if arg.positional {
                return Err(argument(format!("nn {kind}: missing {}", arg.name)));
            }
            continue;
        };
        let valid = matches!(
            (arg.kind, value),
            (Kind::Int, Parameter::Int(_))
                | (Kind::Number, Parameter::Int(_) | Parameter::Float(_))
                | (Kind::IntList, Parameter::IntList(_))
                | (Kind::Bool, Parameter::Bool(_))
                | (Kind::Tensor, Parameter::Tensor(_))
        );
        if !valid {
            return Err(argument(format!(
                "nn {kind}: {} must be {:?}",
                arg.name, arg.kind
            )));
        }
        if let Parameter::Float(f) = value {
            if !f.is_finite() {
                return Err(argument(format!("nn {kind}: {} must be finite", arg.name)));
            }
        }
    }
    Ok(())
}
