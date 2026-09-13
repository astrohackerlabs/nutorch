//! Native tensors: Nu values call the typed core directly, without daemon dispatch.
use nu_engine::command_prelude::*;
use nu_protocol::{CellPathMutation, CustomValue, DeclId, Module, ast::PathMember};
use nutorch_core::{Data, SharedTensor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::Any;
mod nn;
mod table;

fn error(message: impl Into<String>, span: Span) -> ShellError {
    ShellError::Generic(nu_protocol::shell_error::generic::GenericError::new(
        "NuTorch",
        message.into(),
        span,
    ))
}
fn core_error((code, message): nutorch_core::Error, span: Span) -> ShellError {
    error(format!("{code}: {message}"), span)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_serde_cannot_export_or_forge_live_tensors() {
        let tensor = TensorValue(SharedTensor::create(&Data::Int(1), None, false).unwrap());
        assert!(
            serde_json::to_string(&tensor)
                .unwrap_err()
                .to_string()
                .contains("cannot be serialized")
        );
        assert!(
            serde_json::from_str::<TensorValue>("{}")
                .unwrap_err()
                .to_string()
                .contains("cannot be deserialized")
        );
        let forged = r#"{"type":"nutorch.tensor","pointer":123,"handle":"tensor://123"}"#;
        assert!(serde_json::from_str::<Box<dyn CustomValue>>(forged).is_err());
        let wrapped = value(tensor.0.clone(), Span::test_data());
        let cloned = wrapped.clone();
        assert_eq!(cloned.get_type(), Type::Custom("tensor".into()));
        assert!(serde_json::to_string(&wrapped).is_err());
    }
}

#[derive(Debug, Clone)]
pub struct TensorValue(pub SharedTensor);
impl Serialize for TensorValue {
    fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(
            "live tensors cannot be serialized; use torch tolist for data",
        ))
    }
}
impl<'de> Deserialize<'de> for TensorValue {
    fn deserialize<D: Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom(
            "a live tensor cannot be deserialized; use torch tensor to create one",
        ))
    }
}
#[typetag::serde(name = "nutorch.tensor")]
impl CustomValue for TensorValue {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }
    fn type_name(&self) -> String {
        "tensor".into()
    }
    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        let m = self.0.metadata().map_err(|e| core_error(e, span))?;
        // A display summary, not a serialized handle or a data download.
        Ok(Value::string(
            format!(
                "tensor<shape={:?}, dtype={}, device={}, requires_grad={}>",
                m.shape, m.dtype, m.device, m.requires_grad
            ),
            span,
        ))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn update_data_at_cell_path(
        &self,
        _path: &[PathMember],
        _new_val: Value,
        _action: &CellPathMutation,
        head: Span,
    ) -> Result<Value, ShellError> {
        Err(error(
            "tensor cell-path mutation is unsupported; use explicit torch operations",
            head,
        ))
    }
}

fn tensor(value: &Value) -> Result<SharedTensor, ShellError> {
    if let Value::Custom { val, .. } = value {
        if let Some(t) = val.as_any().downcast_ref::<TensorValue>() {
            return Ok(t.0.clone());
        }
    }
    Err(error(
        "expected a native tensor, not a legacy handle or ordinary value",
        value.span(),
    ))
}
fn from_value(value: &Value) -> Result<Data, ShellError> {
    match value {
        Value::Int { val, .. } => Ok(Data::Int(*val)),
        Value::Float { val, .. } => Ok(Data::Float(*val)),
        Value::Bool { val, .. } => Ok(Data::Bool(*val)),
        Value::String { val, .. } if val == "NaN" => Ok(Data::Float(f64::NAN)),
        Value::String { val, .. } if val == "Infinity" => Ok(Data::Float(f64::INFINITY)),
        Value::String { val, .. } if val == "-Infinity" => Ok(Data::Float(f64::NEG_INFINITY)),
        Value::List { vals, .. } => vals
            .iter()
            .map(from_value)
            .collect::<Result<Vec<_>, _>>()
            .map(Data::List),
        _ => Err(error(
            "tensor input must be numeric/boolean scalars or rectangular lists",
            value.span(),
        )),
    }
}
fn to_value(data: Data, span: Span) -> Value {
    match data {
        Data::Int(i) => Value::int(i, span),
        Data::Float(f) => Value::float(f, span),
        Data::Bool(b) => Value::bool(b, span),
        Data::List(xs) => Value::list(xs.into_iter().map(|v| to_value(v, span)).collect(), span),
    }
}
fn value(t: SharedTensor, span: Span) -> Value {
    Value::custom(Box::new(TensorValue(t)), span)
}

const COMMANDS: &[&str] = &[
    "torch",
    "torch tensor",
    "torch shape",
    "torch tolist",
    "torch value",
    "torch ops",
];
pub fn add_context(mut engine: EngineState) -> EngineState {
    let mut working = StateWorkingSet::new(&engine);
    working.enter_scope();
    let first_decl = working.num_decls();
    for name in COMMANDS {
        register_command(&mut working, Box::new(TorchCommand(name)));
    }
    for name in [
        "torch free",
        "torch tensors",
        "torch daemon",
        "torch nu-module",
    ] {
        register_command(&mut working, Box::new(TorchCommand(name)));
    }
    for spec in nutorch_ops::OPS {
        register_command(&mut working, Box::new(table::TableCommand::new(spec)));
    }
    nn::register(&mut working);
    let mut module = Module::new(b"torch".to_vec());
    let mut compatibility = Module::new(b"nutorch".to_vec());
    for index in first_decl..working.num_decls() {
        let id = DeclId::new(index);
        let name = working.get_decl(id).name();
        if name == "torch" {
            module.main = Some(id);
        } else if let Some(relative) = name.strip_prefix("torch ") {
            module.add_decl(relative.as_bytes().to_vec(), id);
        } else if let Some(relative) = name.strip_prefix("nutorch ") {
            compatibility.add_decl(relative.as_bytes().to_vec(), id);
        }
    }
    let compatibility_id = working.add_module("nutorch", compatibility, vec![]);
    module.add_submodule(b"nutorch".to_vec(), compatibility_id);
    working.exit_scope();
    working.add_module("torch", module, vec![]);
    engine
        .merge_delta(working.render())
        .expect("register native torch commands");
    engine
}
#[derive(Clone)]
struct TorchCommand(&'static str);

// Preserve the former module's computational `nutorch ...` aliases, while
// keeping `torch ...` as the canonical command family and help source.
fn register_command(working: &mut StateWorkingSet, command: Box<dyn Command>) {
    if let Some(suffix) = command.name().strip_prefix("torch ") {
        working.add_decl(Box::new(CompatibilityCommand {
            name: format!("nutorch {suffix}"),
            command: command.clone(),
        }));
    }
    working.add_decl(command);
}
#[derive(Clone)]
struct CompatibilityCommand {
    name: String,
    command: Box<dyn Command>,
}
impl Command for CompatibilityCommand {
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        self.command.description()
    }
    fn extra_description(&self) -> &str {
        self.command.extra_description()
    }
    fn signature(&self) -> Signature {
        let mut sig = self.command.signature();
        sig.name = self.name.clone();
        sig
    }
    fn run(
        &self,
        engine: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        self.command.run(engine, stack, call, input)
    }
}
impl Command for TorchCommand {
    fn name(&self) -> &str {
        self.0
    }
    fn description(&self) -> &str {
        match self.0 {
            "torch" => "List the built-in tensor commands.",
            "torch tensor" => "Create a native MPS tensor from data or a dtype/shape/data record.",
            "torch shape" => "Return a tensor shape without downloading data.",
            "torch ops" => "List the built-in tensor operation metadata.",
            "torch free" | "torch tensors" | "torch daemon" | "torch nu-module" => {
                "Explain the retirement of daemon and imported-client controls."
            }
            _ => "Copy tensor data to ordinary shell values, optionally with dtype and shape.",
        }
    }
    fn signature(&self) -> Signature {
        let mut sig = Signature::build(self.0)
            .input_output_types(vec![(Type::Any, Type::Any)])
            .allow_variants_without_examples(true)
            .rest(
                "operands",
                SyntaxShape::Any,
                "Explicit input, or use pipeline input.",
            )
            .category(Category::Custom("torch".into()));
        if self.0 == "torch tensor" {
            sig = sig
                .named("dtype", SyntaxShape::String, "Tensor element type.", None)
                .switch("requires_grad", "Create a tracked autograd leaf.", None)
                .switch("requires-grad", "Create a tracked autograd leaf.", None);
        } else if self.0 == "torch" {
            sig = sig.switch("version", "Show the NuTorch product version.", None);
        } else if matches!(self.0, "torch value" | "torch tolist") {
            sig = sig.switch(
                "meta",
                "Include dtype, shape and data for explicit round-trip creation.",
                None,
            );
        }
        sig
    }
    fn run(
        &self,
        engine: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let span = call.head;
        engine.signals().check(&span)?;
        if matches!(
            self.0,
            "torch free" | "torch tensors" | "torch daemon" | "torch nu-module"
        ) {
            return Err(error(
                "daemon/client controls were removed: native values use shared reference lifetime; inspect them with torch shape, torch value or torch nn info",
                span,
            ));
        }
        if self.0 == "torch" && call.has_flag(engine, stack, "version")? {
            return Ok(
                Value::string(format!("nutorch {}", env!("CARGO_PKG_VERSION")), span)
                    .into_pipeline_data(),
            );
        }
        let mut operands: Vec<Value> = call.rest(engine, stack, 0)?;
        if operands.is_empty() && !matches!(input, PipelineData::Empty) {
            operands.push(input.into_value(span)?);
        }
        let output = match self.0 {
            "torch" | "torch ops" => {
                if !operands.is_empty() {
                    return Err(error(
                        "unsupported native torch command; run torch for built-in commands",
                        span,
                    ));
                }
                if self.0 == "torch" {
                    Value::list(
                        COMMANDS[1..]
                            .iter()
                            .map(|s| (*s).to_owned())
                            .chain(nutorch_ops::OPS.iter().map(|s| format!("torch {}", s.name)))
                            .chain(
                                ["torch nn", "torch forward", "torch step"]
                                    .into_iter()
                                    .map(str::to_owned),
                            )
                            .map(|s| Value::string(s, span))
                            .collect(),
                        span,
                    )
                } else {
                    Value::list(
                        nutorch_ops::OPS
                            .iter()
                            .map(|spec| {
                                let mut record = Record::new();
                                record.push("name", Value::string(spec.name, span));
                                record.push("category", Value::string(spec.category, span));
                                record.push("summary", Value::string(spec.summary, span));
                                record.push("usage", Value::string(spec.usage(), span));
                                Value::record(record, span)
                            })
                            .collect(),
                        span,
                    )
                }
            }
            _ => {
                if operands.len() != 1 {
                    return Err(error(
                        format!("{} expects one input, got {}", self.0, operands.len()),
                        span,
                    ));
                }
                if self.0 == "torch tensor" {
                    let dtype: Option<String> = call.get_flag(engine, stack, "dtype")?;
                    let tracked = call.has_flag(engine, stack, "requires_grad")?
                        || call.has_flag(engine, stack, "requires-grad")?;
                    value(create(&operands[0], dtype.as_deref(), tracked)?, span)
                } else {
                    let tensor = tensor(&operands[0])?;
                    let metadata = tensor.metadata().map_err(|e| core_error(e, span))?;
                    let shape = Value::list(
                        metadata
                            .shape
                            .into_iter()
                            .map(|v| Value::int(v, span))
                            .collect(),
                        span,
                    );
                    if self.0 == "torch shape" {
                        shape
                    } else {
                        let data = to_value(tensor.data().map_err(|e| core_error(e, span))?, span);
                        if call.has_flag(engine, stack, "meta")? {
                            let mut record = Record::new();
                            record.push("dtype", Value::string(metadata.dtype, span));
                            record.push("shape", shape);
                            record.push("data", data);
                            Value::record(record, span)
                        } else {
                            data
                        }
                    }
                }
            }
        };
        engine.signals().check(&span)?;
        Ok(output.into_pipeline_data())
    }
}

fn create(
    input: &Value,
    flag_dtype: Option<&str>,
    tracked: bool,
) -> Result<SharedTensor, ShellError> {
    let span = input.span();
    let (data, dtype, shape) = if let Value::Record { val, .. } = input {
        let data = val
            .get("data")
            .ok_or_else(|| error("tensor record must contain data", span))?;
        let dtype = val.get("dtype").map(Value::as_str).transpose()?;
        let shape = val
            .get("shape")
            .map(|v| {
                v.as_list()?
                    .iter()
                    .map(Value::as_int)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;
        (data, dtype, shape)
    } else {
        (input, None, None)
    };
    if let (Some(a), Some(b)) = (dtype, flag_dtype) {
        if a != b {
            return Err(error(
                format!("envelope dtype {a} conflicts with --dtype {b}"),
                span,
            ));
        }
    }
    let tensor = SharedTensor::create(&from_value(data)?, dtype.or(flag_dtype), tracked)
        .map_err(|e| core_error(e, span))?;
    if let Some(shape) = shape {
        let actual = tensor.metadata().map_err(|e| core_error(e, span))?.shape;
        if actual != shape {
            return Err(error(
                format!("envelope shape {shape:?} does not match data shape {actual:?}"),
                span,
            ));
        }
    }
    Ok(tensor)
}
