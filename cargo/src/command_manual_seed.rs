use nu_plugin::PluginCommand;
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, SyntaxShape, Type
};

use crate::NutorchPlugin;

pub struct CommandManualSeed;

impl PluginCommand for CommandManualSeed {
    type Plugin = NutorchPlugin;

    fn name(&self) -> &str {
        "torch manual_seed"
    }

    fn description(&self) -> &str {
        "Set the random seed for PyTorch operations to ensure reproducibility. (similar to torch.manual_seed() in PyTorch)"
    }

    fn signature(&self) -> Signature {
        Signature::build("torch manual_seed")
            .required(
                "seed",
                SyntaxShape::Int,
                "The seed value for the random number generator",
            )
            .input_output_types(vec![(Type::Nothing, Type::Nothing)])
            .category(Category::Custom("torch".into()))
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Set the random seed to 42 for reproducibility",
            example: "torch manual_seed 42",
            result: None,
        }]
    }

    fn run(
        &self,
        _plugin: &NutorchPlugin,
        _engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        // ── get seed value from required argument ────────────────────────────
        let seed: i64 = call.nth(0).unwrap().as_int()?;

        // ── set the global random seed ────────────────────────────────────────
        tch::manual_seed(seed);

        // ── return empty (operation modifies global state) ────────────────────
        Ok(PipelineData::Empty)
    }
}
