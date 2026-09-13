//! Mode dispatcher trait for external shell integration (e.g., Shannon).
//!
//! When a host binary provides a `ModeDispatcher`, the REPL checks
//! `$env.NUTORCH_MODE` each iteration. If the mode is not "nu", the
//! dispatcher handles the command instead of nushell's parser/evaluator.

use std::collections::HashMap;
use std::path::PathBuf;

/// Trait for dispatching commands to non-nushell modes (e.g., brush, AI).
///
/// The dispatcher receives string env vars (already converted from nushell's
/// typed Values) and returns strings. The REPL handles conversion back to
/// nushell Values.
pub trait ModeDispatcher: Send {
    /// Execute a command in the given mode.
    ///
    /// - `mode`: the active mode name (e.g., "brush", "ai")
    /// - `command`: the raw command string from the user
    /// - `env_vars`: all exported env vars as strings
    /// - `cwd`: the current working directory
    fn execute(
        &mut self,
        mode: &str,
        command: &str,
        env_vars: HashMap<String, String>,
        cwd: PathBuf,
    ) -> ModeResult;
}

/// Result of a mode dispatch execution.
pub struct ModeResult {
    /// Updated environment variables (as strings).
    pub env: HashMap<String, String>,
    /// Updated working directory.
    pub cwd: PathBuf,
    /// Exit code of the command.
    pub exit_code: i32,
}
