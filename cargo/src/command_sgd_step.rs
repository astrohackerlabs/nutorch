use nu_plugin::{PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};
use tch::Tensor;

use crate::NutorchPlugin;
use crate::TENSOR_REGISTRY;


// torch sgd_step  -----------------------------------------------------------
// Performs *in-place* SGD update:  p -= lr * p.grad  (and zeroes the grad).
// Accept a list of tensor-IDs either from the pipeline **or** as the first
// positional argument (but not both).  Returns the *same* list of IDs.
//
// Example usage
//     [$w1 $w2] | torch sgd_step --lr 0.05
//     torch sgd_step [$w1 $w2] --lr 0.05
// ---------------------------------------------------------------------------

pub struct CommandSgdStep;

impl PluginCommand for CommandSgdStep {
    type Plugin = NutorchPlugin;

    fn name(&self) -> &str {
        "torch sgd_step"
    }

    fn description(&self) -> &str {
        "Perform an in-place SGD optimizer step: p -= lr * p.grad for each parameter. (similar to optimizer.step() in PyTorch SGD)"
    }

    fn signature(&self) -> Signature {
        Signature::build("torch sgd_step")
            // list of ids in  -> list of ids out
            .input_output_types(vec![
                (
                    Type::List(Box::new(Type::String)),
                    Type::List(Box::new(Type::String)),
                ),
                (Type::Nothing, Type::List(Box::new(Type::String))),
            ])
            .optional(
                "params",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "List of parameter tensor IDs (if not supplied by pipeline)",
            )
            .named(
                "lr",
                SyntaxShape::Float,
                "Learning-rate (default 0.01)",
                None,
            )
            .category(Category::Custom("torch".into()))
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "SGD step with parameter list piped in",
                example: r#"
let w1 = (torch full [2,2] 1)      # pretend w1.grad is already populated
let w2 = (torch full [2,2] 2)
[$w1, $w2] | torch sgd_step --lr 0.1
"#
                .trim(),
                result: None,
            },
            Example {
                description: "SGD step with parameter list as argument",
                example: r#"
let w1 = (torch full [2,2] 1)
let w2 = (torch full [2,2] 2)
torch sgd_step [$w1, $w2] --lr 0.05
"#
                .trim(),
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
        // Supports: [$params] | torch sgd_step   OR   torch sgd_step [$params]
        let list_from_pipe: Option<Value> = match &input {
            PipelineData::Value(v, _) => Some(v.clone()),
            PipelineData::Empty => None,
            _ => {
                return Err(LabeledError::new("Unsupported input").with_label(
                    "Only Value or Empty pipeline inputs are supported",
                    call.head,
                ))
            }
        };

        let list_from_arg: Option<Value> = call.nth(0);

        // ── validate exactly one input source ─────────────────────────
        match (&list_from_pipe, &list_from_arg) {
            (None, None) => {
                return Err(LabeledError::new("Missing input")
                    .with_label("Provide parameter list via pipeline or argument", call.head));
            }
            (Some(_), Some(_)) => {
                return Err(LabeledError::new("Conflicting input").with_label(
                    "Provide parameter list via pipeline OR argument, not both",
                    call.head,
                ));
            }
            _ => {}
        };

        // ── extract parameter IDs from list ───────────────────────────
        let list_val = list_from_pipe.or(list_from_arg).unwrap();

        let param_ids: Vec<String> = list_val
            .as_list()
            .map_err(|_| {
                LabeledError::new("Invalid input")
                    .with_label("Parameter list must be a list of tensor IDs", call.head)
            })?
            .iter()
            .map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Result<Vec<String>, _>>()?;

        if param_ids.is_empty() {
            return Err(
                LabeledError::new("Invalid input").with_label("Parameter list is empty", call.head)
            );
        }

        // ── extract learning rate (default 0.01) ──────────────────────
        let lr: f64 = call.get_flag("lr")?.unwrap_or(0.01);

        // ── fetch tensors from registry ───────────────────────────────
        let registry = TENSOR_REGISTRY.lock().unwrap();
        let mut tensors: Vec<Tensor> = Vec::new();
        for id in &param_ids {
            match registry.get(id) {
                Some(tensor) => tensors.push(tensor.shallow_clone()),
                None => {
                    return Err(LabeledError::new("Tensor not found")
                        .with_label(format!("Invalid tensor ID: {}", id), call.head))
                }
            }
        }

        // ── perform in-place SGD update: p -= lr * grad ───────────────
        // Use no_grad to disable gradient tracking during parameter updates
        tch::no_grad(|| {
            for mut p in tensors {
                let g = p.grad();
                if g.defined() {
                    // In-place subtraction: p -= lr * grad
                    let before_ptr = p.data_ptr();
                    let r = p.f_sub_(&(g * lr)).unwrap();
                    assert_eq!(before_ptr, r.data_ptr()); // verify in-place operation
                }
            }
        });

        // ── return same parameter IDs for chaining ────────────────────
        let out_vals: Vec<Value> = param_ids
            .iter()
            .map(|id| Value::string(id, call.head))
            .collect();
        Ok(PipelineData::Value(Value::list(out_vals, call.head), None))
    }
}
