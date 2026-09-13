use super::{core_error, error, tensor, to_value, value};
use nu_engine::command_prelude::*;
use nutorch_core::operations::{Output, Parameter, Params, execute};
use nutorch_ops::{Arity, OpSpec, ParamKind, ResultKind};

#[derive(Clone)]
pub(super) struct TableCommand {
    name: String,
    description: String,
    usage: String,
    spec: &'static OpSpec,
}
impl TableCommand {
    pub(super) fn new(spec: &'static OpSpec) -> Self {
        let mut summary = spec.summary.chars();
        let description = format!(
            "{}{}.",
            summary
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default(),
            summary.as_str().trim_end_matches('.')
        );
        Self {
            name: format!("torch {}", spec.name),
            description,
            usage: spec.usage(),
            spec,
        }
    }
}
fn shape(kind: ParamKind) -> SyntaxShape {
    match kind {
        ParamKind::Int => SyntaxShape::Int,
        ParamKind::Float | ParamKind::Scalar => SyntaxShape::Number,
        ParamKind::IntList => SyntaxShape::List(Box::new(SyntaxShape::Int)),
        ParamKind::Bool => SyntaxShape::Boolean,
        ParamKind::Str => SyntaxShape::String,
        ParamKind::TensorOrScalar => SyntaxShape::Any,
    }
}
fn parameter(v: &Value, kind: ParamKind) -> Result<Parameter, ShellError> {
    Ok(match (kind, v) {
        (
            ParamKind::Int | ParamKind::Float | ParamKind::Scalar | ParamKind::TensorOrScalar,
            Value::Int { val, .. },
        ) => Parameter::Int(*val),
        (
            ParamKind::Float | ParamKind::Scalar | ParamKind::TensorOrScalar,
            Value::Float { val, .. },
        ) => Parameter::Float(*val),
        (ParamKind::IntList, Value::List { vals, .. }) => {
            Parameter::IntList(vals.iter().map(Value::as_int).collect::<Result<_, _>>()?)
        }
        (ParamKind::Bool, Value::Bool { val, .. }) => Parameter::Bool(*val),
        (ParamKind::Str, Value::String { val, .. }) => Parameter::Str(val.clone()),
        (ParamKind::TensorOrScalar, Value::Custom { .. }) => Parameter::Tensor(tensor(v)?),
        _ => {
            return Err(error(
                format!("expected parameter of type {kind:?}"),
                v.span(),
            ));
        }
    })
}
impl Command for TableCommand {
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn extra_description(&self) -> &str {
        &self.usage
    }
    fn signature(&self) -> Signature {
        let mut sig = Signature::build(&self.name)
            .input_output_types(vec![(Type::Any, Type::Any)])
            .allow_variants_without_examples(true)
            .rest("operands", SyntaxShape::Any, "Tensors followed by positional parameters; pipeline input supplies the first tensor.")
            .category(Category::Custom("torch".into()));
        for param in self.spec.params.iter().filter(|p| !p.positional) {
            sig = if param.kind == ParamKind::Bool {
                sig.switch(param.name, "Enable this operation option.", None)
            } else {
                sig.named(param.name, shape(param.kind), "Operation parameter.", None)
            };
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
        let mut operands: Vec<Value> = call.rest(engine, stack, 0)?;
        if !matches!(input, PipelineData::Empty) {
            let piped = input.into_value(span)?;
            match piped {
                Value::List { vals, .. } if !matches!(self.spec.tensors, Arity::Exactly(0)) => {
                    operands.splice(0..0, vals).for_each(drop);
                }
                v => operands.insert(0, v),
            }
        }
        if matches!(self.spec.tensors, Arity::AtLeast(_)) {
            operands = operands
                .into_iter()
                .flat_map(|v| match v {
                    Value::List { vals, .. } => vals.into_iter().collect::<Vec<_>>(),
                    v => vec![v],
                })
                .collect();
        }
        let positional = self
            .spec
            .params
            .iter()
            .filter(|p| p.positional)
            .collect::<Vec<_>>();
        let tensor_count = match self.spec.tensors {
            Arity::Exactly(n) => n,
            Arity::AtLeast(n) => {
                if operands.len() < n {
                    return Err(error(
                        format!("{} expects at least {n} tensors", self.name),
                        span,
                    ));
                }
                operands.len()
            }
        };
        if operands.len() != tensor_count + positional.len() {
            return Err(error(
                format!(
                    "{} expects {tensor_count} tensor(s) and {} positional parameter(s), counting pipeline input; got {} operands",
                    self.name,
                    positional.len(),
                    operands.len()
                ),
                span,
            ));
        }
        let tensors = operands[..tensor_count]
            .iter()
            .map(tensor)
            .collect::<Result<Vec<_>, _>>()?;
        let mut params = Params::default();
        for (p, v) in positional.iter().zip(&operands[tensor_count..]) {
            params.values.insert(p.name.into(), parameter(v, p.kind)?);
        }
        for p in self.spec.params.iter().filter(|p| !p.positional) {
            if p.kind == ParamKind::Bool {
                if call.has_flag(engine, stack, p.name)? {
                    params.values.insert(p.name.into(), Parameter::Bool(true));
                }
            } else if let Some(v) = call.get_flag::<Value>(engine, stack, p.name)? {
                params.values.insert(p.name.into(), parameter(&v, p.kind)?);
            }
        }
        let output = match execute(self.spec, &tensors, &params).map_err(|e| core_error(e, span))? {
            Output::Nothing => Value::nothing(span),
            Output::Value(v) => to_value(v, span),
            Output::Tensors(mut tensors) if self.spec.results == ResultKind::Tensors(1) => {
                value(tensors.remove(0), span)
            }
            Output::Tensors(tensors) => {
                Value::list(tensors.into_iter().map(|t| value(t, span)).collect(), span)
            }
        };
        engine.signals().check(&span)?;
        Ok(output.into_pipeline_data())
    }
}
