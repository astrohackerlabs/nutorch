use nu_plugin::PluginCommand;
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type};
use tch::Device;

use crate::tensor_to_value;
use crate::NutorchPlugin;
use crate::TENSOR_REGISTRY;

// Command to convert tensor to Nushell data structure (value)
pub struct CommandValue;

impl PluginCommand for CommandValue {
    type Plugin = NutorchPlugin;

    fn name(&self) -> &str {
        "torch value"
    }

    fn description(&self) -> &str {
        "Convert a tensor to a Nushell value (nested list or scalar). (similar to tensor.tolist() or tensor.numpy() in PyTorch)"
    }

    fn signature(&self) -> Signature {
        Signature::build("torch value")
            .input_output_types(vec![
                (Type::String, Type::Any),  // tensor id via pipeline
                (Type::Nothing, Type::Any), // tensor id via arg
            ])
            .optional(
                "tensor_id",
                SyntaxShape::String,
                "ID of the tensor (if not supplied by pipeline)",
            )
            .category(Category::Custom("torch".into()))
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Convert a 1D tensor to a Nushell value via pipeline",
                example: "torch linspace 0.0 1.0 4 | torch value",
                result: None,
            },
            Example {
                description: "Convert a 2D tensor to nested values via argument",
                example: "let t = (torch linspace 0.0 1.0 4 | torch repeat 2 2); torch value $t",
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
        // ── dual input: pipeline OR argument (not both) ───────────────
        // Supports: $t | torch value   OR   torch value $t
        let piped = match input {
            PipelineData::Empty => None,
            PipelineData::Value(v, _) => Some(v),
            _ => {
                return Err(LabeledError::new("Unsupported input")
                    .with_label("Only Empty or single Value inputs are supported", call.head))
            }
        };

        let arg0 = call.nth(0);

        // ── validate exactly one input source ─────────────────────────
        let tensor_id = match (piped, arg0) {
            (Some(_), Some(_)) => {
                return Err(LabeledError::new("Conflicting input").with_label(
                    "Provide tensor ID via pipeline OR argument, not both",
                    call.head,
                ))
            }
            (None, None) => {
                return Err(LabeledError::new("Missing input")
                    .with_label("Provide tensor ID via pipeline or argument", call.head))
            }
            (Some(v), None) => v.as_str()?.to_string(),
            (None, Some(a)) => a.as_str()?.to_string(),
        };

        // ── fetch tensor from registry ────────────────────────────────
        let registry = TENSOR_REGISTRY.lock().unwrap();
        let tensor = registry.get(&tensor_id).ok_or_else(|| {
            LabeledError::new("Tensor not found").with_label("Invalid tensor ID", call.head)
        })?;

        // ── ensure tensor is on CPU ───────────────────────────────────
        // Must move to CPU before accessing data
        let tensor = tensor.to_device(Device::Cpu);

        // ── convert tensor to Nushell value ───────────────────────────
        let span = call.head;
        let value = tensor_to_value(&tensor, span)?;

        Ok(PipelineData::Value(value, None))
    }
}
