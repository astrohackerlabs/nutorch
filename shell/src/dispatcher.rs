use std::{collections::HashMap, path::PathBuf};

use nu_cli::{ModeDispatcher, ModeResult};

pub const AI_STUB_LINE: &str =
    "AI mode is unfinished; input was not executed. Shift+Tab returns to Nushell.";

/// The shell evaluates normal Nushell itself. Alternate input is never code.
pub struct NuTorchDispatcher;

impl ModeDispatcher for NuTorchDispatcher {
    fn execute(
        &mut self,
        mode: &str,
        _command: &str,
        _env: HashMap<String, String>,
        cwd: PathBuf,
    ) -> ModeResult {
        if mode == "ai" {
            println!("{AI_STUB_LINE}");
        } else {
            eprintln!("Unknown NuTorch mode: {mode}");
        }
        ModeResult {
            // No environment updates: preserve Nushell's typed values.
            env: HashMap::new(),
            cwd,
            exit_code: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternate_input_has_no_execution_or_environment_side_effects() {
        let temp = tempfile::tempdir().unwrap();
        let sentinel = temp.path().join("executed");
        let input = format!("touch '{}'", sentinel.display());
        let mut dispatcher = NuTorchDispatcher;
        for mode in ["ai", "zsh", "unknown"] {
            let result = dispatcher.execute(
                mode,
                &input,
                HashMap::from([("PATH".into(), "unchanged".into())]),
                temp.path().to_path_buf(),
            );
            assert_ne!(result.exit_code, 0);
            assert_eq!(result.cwd, temp.path());
            assert!(result.env.is_empty());
            assert!(!sentinel.exists());
        }
    }
}
