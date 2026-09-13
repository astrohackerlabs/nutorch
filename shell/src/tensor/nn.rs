use super::*;
use nutorch_core::{
    nn::{
        SharedModule, SharedOptimizer,
        schema::{self, Kind},
    },
    operations::{Parameter, Params},
};

macro_rules! native_value {
    ($name:ident, $inner:ty, $tag:literal, $typename:literal) => {
        #[derive(Clone, Debug)]
        struct $name($inner);
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom(concat!(
                    "live ",
                    $typename,
                    " values cannot be serialized; save module state explicitly"
                )))
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
                Err(serde::de::Error::custom(concat!(
                    "live ",
                    $typename,
                    " values cannot be deserialized"
                )))
            }
        }
        #[typetag::serde(name = $tag)]
        impl CustomValue for $name {
            fn clone_value(&self, span: Span) -> Value {
                Value::custom(Box::new(self.clone()), span)
            }
            fn type_name(&self) -> String {
                $typename.into()
            }
            fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
                Ok(Value::string(
                    format!(
                        "{}<{}>",
                        $typename,
                        self.0
                            .describe()
                            .map_err(|e| core_error(e, span))?
                            .join(", ")
                            .chars()
                            .take(480)
                            .collect::<String>()
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
                _: &[PathMember],
                _: Value,
                _: &CellPathMutation,
                span: Span,
            ) -> Result<Value, ShellError> {
                Err(error(
                    "use explicit torch nn operations to modify native state",
                    span,
                ))
            }
        }
    };
}
native_value!(ModuleValue, SharedModule, "nutorch.module", "module");
native_value!(
    OptimizerValue,
    SharedOptimizer,
    "nutorch.optimizer",
    "optimizer"
);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn live_modules_and_optimizers_reject_raw_serde_and_forgery() {
        let mut params = Params::default();
        params
            .values
            .insert("in_features".into(), Parameter::Int(1));
        params
            .values
            .insert("out_features".into(), Parameter::Int(1));
        let module = SharedModule::create("linear", &params).unwrap();
        let optimizer = SharedOptimizer::create("adam", &module, &Params::default()).unwrap();
        for value in [
            Value::custom(Box::new(ModuleValue(module)), Span::test_data()),
            Value::custom(Box::new(OptimizerValue(optimizer)), Span::test_data()),
        ] {
            assert!(serde_json::to_string(&value).is_err());
            let alias = value.clone();
            assert_eq!(alias.get_type(), value.get_type());
            if let Value::Custom { val, .. } = alias {
                assert!(
                    val.to_base_value(Span::test_data())
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .len()
                        < 512
                );
            }
        }
        for tag in ["nutorch.module", "nutorch.optimizer"] {
            let forged = serde_json::json!({"type": tag, "pointer": 123, "handle": "nn://fake"});
            assert!(serde_json::from_value::<Box<dyn CustomValue>>(forged).is_err());
        }
    }
}
fn module(value: &Value) -> Result<SharedModule, ShellError> {
    if let Value::Custom { val, .. } = value {
        if let Some(m) = val.as_any().downcast_ref::<ModuleValue>() {
            return Ok(m.0.clone());
        }
    }
    Err(error("expected a native module", value.span()))
}
fn optimizer(value: &Value) -> Result<SharedOptimizer, ShellError> {
    if let Value::Custom { val, .. } = value {
        if let Some(m) = val.as_any().downcast_ref::<OptimizerValue>() {
            return Ok(m.0.clone());
        }
    }
    Err(error("expected a native optimizer", value.span()))
}
const METHODS: &[&str] = &[
    "parameters",
    "info",
    "zero_grad",
    "set_lr",
    "save",
    "load",
    "train",
    "eval",
];
pub(super) fn register(working: &mut StateWorkingSet) {
    for kind in schema::MODULES
        .iter()
        .chain(schema::OPTIMIZERS)
        .chain(METHODS)
    {
        register_command(
            working,
            Box::new(NnCommand {
                name: format!("torch nn {kind}"),
                kind,
            }),
        );
    }
    for (name, kind) in [
        ("torch nn", "nn"),
        ("torch forward", "forward"),
        ("torch step", "step"),
    ] {
        register_command(
            working,
            Box::new(NnCommand {
                name: name.into(),
                kind,
            }),
        );
    }
}
#[derive(Clone)]
struct NnCommand {
    name: String,
    kind: &'static str,
}
fn shape(kind: Kind) -> SyntaxShape {
    match kind {
        Kind::Int => SyntaxShape::Int,
        Kind::Number => SyntaxShape::Number,
        Kind::IntList => SyntaxShape::List(Box::new(SyntaxShape::Int)),
        Kind::Bool => SyntaxShape::Boolean,
        Kind::Tensor => SyntaxShape::Any,
    }
}
fn parameter(v: &Value, kind: Kind) -> Result<Parameter, ShellError> {
    Ok(match kind {
        Kind::Int => Parameter::Int(v.as_int()?),
        Kind::Number => match v {
            Value::Int { val, .. } => Parameter::Int(*val),
            Value::Float { val, .. } => Parameter::Float(*val),
            _ => return Err(error("expected a number", v.span())),
        },
        Kind::IntList => Parameter::IntList(
            v.as_list()?
                .iter()
                .map(Value::as_int)
                .collect::<Result<_, _>>()?,
        ),
        Kind::Bool => Parameter::Bool(v.as_bool()?),
        Kind::Tensor => Parameter::Tensor(tensor(v)?),
    })
}
impl Command for NnCommand {
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        match self.kind {
            "nn" => "List native neural network constructors and methods.",
            "forward" => "Evaluate a native module on a tensor.",
            "step" => "Update parameters through a native optimizer.",
            "parameters" => "Return live tensor aliases of module parameters.",
            "info" => "Describe a native module or optimizer.",
            "zero_grad" => "Clear the gradients of a module or optimizer.",
            "set_lr" => "Set the learning rate shared by optimizer aliases.",
            "save" => "Save module parameters and buffers to a safetensors state file.",
            "load" => "Load a safetensors state file into existing module storage.",
            "train" => "Enable training recursively through shared child modules.",
            "eval" => "Enable evaluation recursively through shared child modules.",
            "sequential" => "Compose shared child modules in order.",
            kind if schema::OPTIMIZERS.contains(&kind) => {
                "Create a native optimizer owning shared module parameters."
            }
            _ => "Create a native neural network module.",
        }
    }
    fn signature(&self) -> Signature {
        let mut sig = Signature::build(&self.name)
            .input_output_types(vec![(Type::Any, Type::Any)])
            .allow_variants_without_examples(true)
            .rest(
                "operands",
                SyntaxShape::Any,
                "Constructor arguments or native values; a pipeline may supply the first input.",
            )
            .category(Category::Custom("torch".into()));
        if let Ok(args) = schema::arguments(self.kind) {
            for arg in args.iter().filter(|a| !a.positional) {
                let mut names = vec![arg.name.to_owned()];
                if arg.name.contains('_') {
                    names.push(arg.name.replace('_', "-"));
                }
                for name in names {
                    sig = if matches!(arg.kind, Kind::Bool) {
                        sig.switch(name, "Enable this constructor option.", None)
                    } else {
                        sig.named(name, shape(arg.kind), "Constructor parameter.", None)
                    };
                }
            }
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
        let mut args: Vec<Value> = call.rest(engine, stack, 0)?;
        if !matches!(input, PipelineData::Empty) {
            let piped = input.into_value(span)?;
            if self.kind == "forward" {
                args.push(piped);
            } else if self.kind == "sequential" {
                match piped {
                    Value::List { vals, .. } => args.splice(0..0, vals).for_each(drop),
                    v => args.insert(0, v),
                }
            } else {
                args.insert(0, piped);
            }
        }
        if self.kind == "sequential" {
            args = args
                .into_iter()
                .flat_map(|v| match v {
                    Value::List { vals, .. } => vals.into_iter().collect::<Vec<_>>(),
                    v => vec![v],
                })
                .collect();
        }
        let require = |n: usize| -> Result<(), ShellError> {
            if args.len() == n {
                Ok(())
            } else {
                Err(error(
                    format!("{} expects {n} operand(s), got {}", self.name, args.len()),
                    span,
                ))
            }
        };
        let output = match self.kind {
            "nn" => {
                require(0)?;
                Value::list(
                    schema::MODULES
                        .iter()
                        .chain(schema::OPTIMIZERS)
                        .chain(METHODS)
                        .map(|s| Value::string(*s, span))
                        .collect(),
                    span,
                )
            }
            "forward" => {
                require(2)?;
                value(
                    module(&args[0])?
                        .forward(&tensor(&args[1])?)
                        .map_err(|e| core_error(e, span))?,
                    span,
                )
            }
            "step" => {
                require(1)?;
                optimizer(&args[0])?
                    .step()
                    .map_err(|e| core_error(e, span))?;
                Value::nothing(span)
            }
            "sequential" => {
                let children = args.iter().map(module).collect::<Result<Vec<_>, _>>()?;
                Value::custom(
                    Box::new(ModuleValue(
                        SharedModule::sequential(children).map_err(|e| core_error(e, span))?,
                    )),
                    span,
                )
            }
            "parameters" => {
                require(1)?;
                Value::list(
                    module(&args[0])?
                        .parameters()
                        .map_err(|e| core_error(e, span))?
                        .into_iter()
                        .map(|t| value(t, span))
                        .collect(),
                    span,
                )
            }
            "info" => {
                require(1)?;
                let lines = if let Ok(m) = module(&args[0]) {
                    m.describe()
                } else {
                    optimizer(&args[0])?.describe()
                }
                .map_err(|e| core_error(e, span))?;
                let mut record = Record::new();
                for line in lines {
                    if let Some((key, val)) = line.split_once(": ") {
                        record.push(key, Value::string(val, span));
                    }
                }
                Value::record(record, span)
            }
            "zero_grad" => {
                require(1)?;
                if let Ok(m) = module(&args[0]) {
                    m.zero_grad()
                } else {
                    optimizer(&args[0])?.zero_grad()
                }
                .map_err(|e| core_error(e, span))?;
                Value::nothing(span)
            }
            "set_lr" => {
                require(2)?;
                optimizer(&args[0])?
                    .set_lr(args[1].as_float()?)
                    .map_err(|e| core_error(e, span))?;
                Value::nothing(span)
            }
            "train" | "eval" => {
                require(1)?;
                module(&args[0])?
                    .set_training(self.kind == "train")
                    .map_err(|e| core_error(e, span))?;
                Value::nothing(span)
            }
            "save" | "load" => {
                require(2)?;
                let raw = args[1].as_str()?;
                let path = nu_path::expand_path_with(raw, engine.cwd(Some(stack))?, true);
                let module = module(&args[0])?;
                if self.kind == "save" {
                    module.save(&path)
                } else {
                    module.load(&path)
                }
                .map_err(|e| core_error(e, span))?;
                let mut receipt = Record::new();
                receipt.push(
                    if self.kind == "save" {
                        "saved"
                    } else {
                        "loaded"
                    },
                    Value::string(path.to_string_lossy(), span),
                );
                Value::record(receipt, span)
            }
            kind => {
                let schema = schema::arguments(kind).map_err(|e| core_error(e, span))?;
                let is_optimizer = schema::OPTIMIZERS.contains(&kind);
                let offset = usize::from(is_optimizer);
                require(offset + schema.iter().filter(|a| a.positional).count())?;
                let mut params = Params::default();
                for (arg, v) in schema.iter().filter(|a| a.positional).zip(&args[offset..]) {
                    params
                        .values
                        .insert(arg.name.into(), parameter(v, arg.kind)?);
                }
                for arg in schema.iter().filter(|a| !a.positional) {
                    let mut names = vec![arg.name.to_owned()];
                    if arg.name.contains('_') {
                        names.push(arg.name.replace('_', "-"));
                    }
                    for name in names {
                        let val = if matches!(arg.kind, Kind::Bool) {
                            call.has_flag(engine, stack, &name)?
                                .then_some(Parameter::Bool(true))
                        } else {
                            call.get_flag::<Value>(engine, stack, &name)?
                                .map(|v| parameter(&v, arg.kind))
                                .transpose()?
                        };
                        if let Some(val) = val {
                            if params.values.insert(arg.name.into(), val).is_some() {
                                return Err(error(
                                    format!("duplicate spellings for {}", arg.name),
                                    span,
                                ));
                            }
                        }
                    }
                }
                if is_optimizer {
                    Value::custom(
                        Box::new(OptimizerValue(
                            SharedOptimizer::create(kind, &module(&args[0])?, &params)
                                .map_err(|e| core_error(e, span))?,
                        )),
                        span,
                    )
                } else {
                    Value::custom(
                        Box::new(ModuleValue(
                            SharedModule::create(kind, &params).map_err(|e| core_error(e, span))?,
                        )),
                        span,
                    )
                }
            }
        };
        engine.signals().check(&span)?;
        Ok(output.into_pipeline_data())
    }
}
