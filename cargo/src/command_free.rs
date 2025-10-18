use nu_plugin::{PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type, Value,
};

use crate::NutorchPlugin;
use crate::TENSOR_REGISTRY;

// torch free  ---------------------------------------------------------------
// Explicitly drop tensors from the global registry so their memory can be
// reclaimed.  Accepts IDs via pipeline OR as the first positional argument
// (string or list of strings) but not both.
//
//     $id  | torch free
//     [$id1 $id2] | torch free
//     torch free $id
//     torch free [$id1 $id2]
//
// Returns the list of IDs that were successfully removed.
// ---------------------------------------------------------------------------
pub struct CommandFree;

impl PluginCommand for CommandFree {
    type Plugin = NutorchPlugin;

    fn name(&self) -> &str {
        "torch free"
    }

    fn description(&self) -> &str {
        "Remove tensor(s) from the internal registry, freeing their memory. (similar to del tensor in Python)"
    }

    fn signature(&self) -> Signature {
        Signature::build("torch free")
            .input_output_types(vec![
                (Type::String, Type::List(Box::new(Type::String))),
                (
                    Type::List(Box::new(Type::String)),
                    Type::List(Box::new(Type::String)),
                ),
                (Type::Nothing, Type::List(Box::new(Type::String))),
            ])
            .optional(
                "tensor_ids",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "Tensor ID or list of IDs to free (if not provided by pipeline)",
            )
            .category(Category::Custom("torch".into()))
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Free a single tensor via pipeline",
                example: r#"
let t = (torch full [1000] 1)
$t | torch free
"#
                .trim(),
                result: None,
            },
            Example {
                description: "Free several tensors in one call",
                example: r#"
let a = (torch randn [1000 1000])
let b = (torch randn [1000 1000])
torch free [$a $b]
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
        // Supports: $t | torch free   OR   [$ts] | torch free
        //       OR  torch free $t     OR   torch free [$ts]
        let piped: Option<Value> = match &input {
            PipelineData::Value(v, _) => Some(v.clone()),
            PipelineData::Empty => None,
            _ => {
                return Err(LabeledError::new("Unsupported input")
                    .with_label("Only Empty or Value inputs accepted", call.head))
            }
        };
        let arg0: Option<Value> = call.nth(0);

        // ── validate exactly one input source ─────────────────────────
        match (&piped, &arg0) {
            (None, None) => {
                return Err(LabeledError::new("Missing input")
                    .with_label("Provide tensor ID(s) via pipeline or argument", call.head))
            }
            (Some(_), Some(_)) => {
                return Err(LabeledError::new("Conflicting input")
                    .with_label("Provide IDs via pipeline OR argument, not both", call.head))
            }
            _ => {}
        }

        // ── extract tensor IDs (single or list) ───────────────────────
        let ids_val = piped.or(arg0).unwrap();

        // Accept single string or list-of-strings
        let ids: Vec<String> = if let Ok(list) = ids_val.as_list() {
            list.iter()
                .map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![ids_val.as_str()?.to_string()]
        };

        if ids.is_empty() {
            return Err(
                LabeledError::new("Empty list").with_label("No tensor IDs supplied", call.head)
            );
        }

        // ── remove tensors from registry ──────────────────────────────
        let mut reg = TENSOR_REGISTRY.lock().unwrap();
        let mut freed: Vec<Value> = Vec::new();

        for id in ids {
            if reg.remove(&id).is_some() {
                // Entry removed successfully; add to return list
                freed.push(Value::string(id, call.head));
            } else {
                return Err(LabeledError::new("Tensor not found")
                    .with_label(format!("Invalid tensor ID: {id}"), call.head));
            }
        }

        // ── return list of freed tensor IDs ───────────────────────────
        Ok(PipelineData::Value(Value::list(freed, call.head), None))
    }
}
