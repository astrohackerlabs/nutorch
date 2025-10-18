use nu_plugin::PluginCommand;
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value};
use uuid::Uuid;

use crate::NutorchPlugin;
use crate::TENSOR_REGISTRY;

// torch sin  ---------------------------------------------------------------
//
// Apply sine function element-wise to a tensor: y = sin(x)
// Accept the tensor ID either from the pipeline or as a single argument.
// -------------------------------------------------------------------------
pub struct CommandSin;

impl PluginCommand for CommandSin {
    type Plugin = NutorchPlugin;

    fn name(&self) -> &str {
        "torch sin"
    }

    fn description(&self) -> &str {
        "Apply sine function element-wise to a tensor (similar to torch.sin)"
    }

    fn signature(&self) -> Signature {
        Signature::build("torch sin")
            .input_output_types(vec![
                (Type::String, Type::String),  // pipeline-in
                (Type::Nothing, Type::String), // arg-in
            ])
            .optional(
                "tensor_id",
                SyntaxShape::String,
                "ID of the tensor (if not provided via pipeline)",
            )
            .category(Category::Custom("torch".into()))
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Apply sine to a tensor via pipeline",
                example: "let t = (torch tensor [0 1.5708 3.1416]); $t | torch sin | torch value",
                result: None,
            },
            Example {
                description: "Apply sine to a tensor via argument",
                example: "let t = (torch tensor [0 1.5708 3.1416]); torch sin $t | torch value",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &NutorchPlugin,
        _engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        // -------- figure out where the tensor ID comes from ----------------
        let piped = match input {
            PipelineData::Empty => None,
            PipelineData::Value(v, _span) => Some(v),
            _ => {
                return Err(LabeledError::new("Unsupported input")
                    .with_label("Only Empty or single Value inputs are supported", call.head))
            }
        };
        let arg0 = call.nth(0);

        let tensor_id = match (piped, arg0) {
            (Some(_), Some(_)) => {
                return Err(LabeledError::new("Conflicting input").with_label(
                    "Provide tensor ID either via pipeline OR argument, not both",
                    call.head,
                ))
            }
            (None, None) => {
                return Err(LabeledError::new("Missing input").with_label(
                    "Tensor ID must be supplied via pipeline or argument",
                    call.head,
                ))
            }
            (Some(v), None) => v.as_str().map(|s| s.to_string()).map_err(|_| {
                LabeledError::new("Invalid input")
                    .with_label("Pipeline input must be a tensor ID (string)", call.head)
            })?,
            (None, Some(a)) => a.as_str().map(|s| s.to_string()).map_err(|_| {
                LabeledError::new("Invalid input")
                    .with_label("Argument must be a tensor ID (string)", call.head)
            })?,
        };

        // -------- fetch tensor from registry -------------------------------
        let mut reg = TENSOR_REGISTRY.lock().unwrap();
        let tensor = reg
            .get(&tensor_id)
            .ok_or_else(|| {
                LabeledError::new("Tensor not found").with_label("Invalid tensor ID", call.head)
            })?
            .shallow_clone();

        // -------- apply sine operation -------------------------------------
        let result_tensor = tensor.sin();

        // -------- store & return -------------------------------------------
        let new_id = Uuid::new_v4().to_string();
        reg.insert(new_id.clone(), result_tensor);
        Ok(PipelineData::Value(Value::string(new_id, call.head), None))
    }
}
