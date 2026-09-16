//! Shared presentation for the external NuTorch CLI help surfaces.
use std::fmt::Write;

pub const HELP_SECTION_COLOR: &str = "\x1b[32m";
pub const HELP_FLAG_COLOR: &str = "\x1b[36m";
pub const HELP_TYPE_COLOR: &str = "\x1b[94m";
pub const HELP_DESC_COLOR: &str = "\x1b[2;39m";
pub const HELP_SUBCMD_COLOR: &str = "\x1b[96m";
pub const DEFAULT_COLOR: &str = "\x1b[39m";
pub const RESET_COLOR: &str = "\x1b[0m";

pub fn start(usages: &[&str]) -> String {
    let mut output =
        String::from("NuTorch — the Astrohacker command shell, powered by Nushell.\n\nUsage:\n");
    for usage in usages {
        writeln!(output, "  {usage}").unwrap();
    }
    output.push('\n');
    output
}

pub fn section(output: &mut String, name: &str) {
    writeln!(output, "{HELP_SECTION_COLOR}{name}:{RESET_COLOR}").unwrap();
}

pub fn description(output: &mut String, text: &str, indent: usize) {
    writeln!(output, "{:indent$}{HELP_DESC_COLOR}{text}{RESET_COLOR}", "").unwrap();
}

pub fn flag(output: &mut String, short: Option<char>, long: &str, value: Option<&str>) {
    output.push_str("  ");
    if let Some(short) = short {
        write!(output, "{HELP_FLAG_COLOR}-{short}{RESET_COLOR}").unwrap();
        if !long.is_empty() {
            write!(output, "{DEFAULT_COLOR},{RESET_COLOR} ").unwrap();
        }
    }
    if !long.is_empty() {
        write!(output, "{HELP_FLAG_COLOR}--{long}{RESET_COLOR}").unwrap();
    }
    if let Some(value) = value {
        write!(output, " <{HELP_TYPE_COLOR}{value}{RESET_COLOR}>").unwrap();
    }
    output.push('\n');
}

pub fn example(output: &mut String, text: &str) {
    writeln!(
        output,
        "      {HELP_DESC_COLOR}Example: {RESET_COLOR}{text}"
    )
    .unwrap();
}

pub fn sync_help() -> String {
    let mut output = start(&["nutorch sync"]);
    section(&mut output, "Description");
    for text in [
        "Queue exported environment variables in the enclosing NuTorch.",
        "Success means queued; updates apply at its next prompt.",
        "Missing keys are not deleted.",
    ] {
        description(&mut output, text, 2);
    }
    output.push_str("\nOptions:\n\n");
    section(&mut output, "General");
    flag(&mut output, Some('h'), "help", None);
    description(&mut output, "show this help message", 6);
    example(&mut output, "nutorch sync --help");
    output
}
