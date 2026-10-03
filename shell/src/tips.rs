//! Built-in usage tips for Nushell and native tensor commands.
//!
//! `nutorch tip` is both an early CLI, before the REPL starts, and an
//! in-process command of the same name. The early path does not read
//! `config.nu`. When coloring is on it builds a highlight-only engine and
//! does not evaluate the tip.

use nu_engine::command_prelude::*;
use reedline::Highlighter;
use std::io::IsTerminal;
use std::sync::{Arc, OnceLock};

pub const META_LINE: &str = "Type nutorch tip for more tips.";

pub const NUSHELL_CATEGORIES: &[&str] = &[
    "thinking-in-nu",
    "pipelines",
    "types",
    "strings",
    "lists",
    "records",
    "tables",
    "navigating-data",
    "variables",
    "operators",
    "control-flow",
    "filters",
    "custom-commands",
    "modules",
    "scripts",
    "environment",
    "configuration",
    "moving-around",
    "loading-data",
    "regex",
    "jobs",
    "aliases",
    "externs",
    "line-editor",
    "parallelism",
    "metadata",
    "math",
    "path",
    "date",
    "conversions",
    "random",
    "bits",
    "bytes",
    "system",
    "history",
    "hash",
    "generators",
    "network",
];

pub const TORCH_CATEGORIES: &[&str] = &[
    "tensors",
    "creation",
    "pointwise",
    "reduction",
    "shape",
    "linalg",
    "loss",
    "comparison",
    "utility",
    "autograd",
    "neural-networks",
];

pub const LEVELS: &[&str] = &["intro", "basic", "intermediate", "advanced"];

const REQUIRED_ADVANCED: &[&str] = &[
    "pipelines",
    "custom-commands",
    "modules",
    "tensors",
    "pointwise",
    "autograd",
    "neural-networks",
];

const DENY_WORDS: &[&str] = &[
    "sleep",
    "input",
    "exit",
    "rm",
    "start",
    "plugin",
    "polars",
    "dataframe",
];

const DENY_PHRASES: &[&str] = &[
    "http get",
    "http post",
    "http put",
    "http patch",
    "http delete",
    "http head",
];

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tip {
    pub kind: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub level: String,
    pub body: String,
}

pub fn catalog() -> &'static [Tip] {
    static TIPS: OnceLock<Vec<Tip>> = OnceLock::new();
    TIPS.get_or_init(|| {
        let tips: Vec<Tip> =
            serde_json::from_str(include_str!("../tips/catalog.json")).expect("parse tip catalog");
        let problems = violations(&tips);
        if !problems.is_empty() {
            panic!("tip catalog is invalid:\n{}", problems.join("\n"));
        }
        tips
    })
}

pub fn choose<T>(matches: &[T], index: u64) -> &T {
    &matches[(index % matches.len() as u64) as usize]
}

pub fn render(tip: &Tip) -> String {
    format!(
        "Tip · {} · {} · {}\n{}\n",
        tip.kind,
        tip.r#type,
        tip.level,
        tip.body.trim_end_matches('\n')
    )
}

/// Whether to color a tip body written to `destination_is_terminal`.
///
/// `UseAnsiColoring::get` always inspects stdout. The intro is written to
/// stderr, so Auto takes the stream the caller names.
pub fn coloring_enabled(
    engine: &EngineState,
    stack: &Stack,
    destination_is_terminal: bool,
) -> bool {
    match stack.get_config(engine).use_ansi_coloring {
        nu_protocol::UseAnsiColoring::True => true,
        nu_protocol::UseAnsiColoring::False => false,
        nu_protocol::UseAnsiColoring::Auto => auto_color(engine, destination_is_terminal),
    }
}

fn auto_color(engine: &EngineState, destination_is_terminal: bool) -> bool {
    let env_value = |name: &str| {
        engine
            .get_env_var(name)
            .and_then(|value| value.coerce_bool().ok())
            .unwrap_or(false)
    };

    if env_value("force_color") {
        return true;
    }
    if env_value("no_color") {
        return false;
    }
    if let Some(cli_color) = engine.get_env_var("clicolor")
        && let Ok(cli_color) = cli_color.coerce_bool()
    {
        return cli_color;
    }
    if let Some(term) = engine.get_env_var("term")
        && term.as_str().ok() == Some("dumb")
    {
        return false;
    }
    destination_is_terminal
}

/// Highlight each source line. The caller decides whether color is on.
pub fn highlight_body(engine: &EngineState, stack: &Stack, body: &str) -> String {
    let highlighter = nu_cli::NuHighlighter::new(Arc::new(engine.clone()), Arc::new(stack.clone()));
    body.split('\n')
        .map(|line| Highlighter::highlight(&highlighter, line, 0).render_simple())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Keep a plain `render` result, coloring only its body when the config allows.
pub fn present(
    engine: &EngineState,
    stack: &Stack,
    rendered: &str,
    destination_is_terminal: bool,
) -> String {
    if rendered.starts_with("Tip · ") && coloring_enabled(engine, stack, destination_is_terminal) {
        highlight_rendered(engine, stack, rendered)
    } else {
        rendered.to_string()
    }
}

fn highlight_rendered(engine: &EngineState, stack: &Stack, rendered: &str) -> String {
    let rendered = rendered.trim_end_matches('\n');
    let Some((label, body)) = rendered.split_once('\n') else {
        return format!("{rendered}\n");
    };
    format!("{label}\n{}\n", highlight_body(engine, stack, body))
}

pub fn query(
    kind: Option<&str>,
    category: Option<&str>,
    level: Option<&str>,
) -> Result<Vec<Tip>, String> {
    let mut echoed = Vec::new();
    if let Some(kind) = kind {
        echoed.push(format!("--kind {kind}"));
    }
    if let Some(category) = category {
        echoed.push(format!("--type {category}"));
    }
    if let Some(level) = level {
        echoed.push(format!("--level {level}"));
    }
    filter(kind, category, level, &echoed)
}

pub fn pick_intro() -> Tip {
    let matches: Vec<Tip> = catalog()
        .iter()
        .filter(|tip| tip.level == "intro")
        .cloned()
        .collect();
    choose(&matches, random_index(matches.len())).clone()
}

pub fn help_text() -> String {
    let mut output = crate::help::start(&[
        "nutorch tip",
        "nutorch tip --kind <kind>",
        "nutorch tip --type <category>",
        "nutorch tip --level <level>",
    ]);
    crate::help::section(&mut output, "Description");
    for text in [
        "Print one random usage tip from the built-in catalog.",
        "Nushell tips also run in stock Nushell.",
        "Torch tips run in NuTorch after use torch.",
        "pipes is an alias for pipelines.",
        "Filters combine. Kinds: nushell, torch.",
        "Levels: intro, basic, intermediate, advanced.",
    ] {
        crate::help::description(&mut output, text, 2);
    }
    output.push_str("\nOptions:\n\n");
    crate::help::section(&mut output, "Filters");
    crate::help::flag(&mut output, None, "kind", Some("nushell|torch"));
    crate::help::description(&mut output, "Limit tips to nushell or torch.", 6);
    crate::help::example(&mut output, "nutorch tip --kind torch");
    crate::help::flag(&mut output, None, "type", Some("category"));
    crate::help::description(
        &mut output,
        "Limit tips to one category. pipes is an alias for pipelines.",
        6,
    );
    crate::help::example(&mut output, "nutorch tip --type pipes");
    crate::help::flag(&mut output, None, "level", Some("level"));
    crate::help::description(
        &mut output,
        "Limit tips to intro, basic, intermediate, or advanced.",
        6,
    );
    crate::help::example(&mut output, "nutorch tip --level advanced");
    output.push('\n');
    crate::help::section(&mut output, "General");
    crate::help::flag(&mut output, Some('h'), "help", None);
    crate::help::description(&mut output, "Show this help message.", 6);
    crate::help::example(&mut output, "nutorch tip --help");
    output.push('\n');
    crate::help::section(&mut output, "Categories");
    for category in NUSHELL_CATEGORIES.iter().chain(TORCH_CATEGORIES) {
        crate::help::description(&mut output, category, 2);
    }
    output
}

fn help_extra() -> &'static str {
    static TEXT: OnceLock<String> = OnceLock::new();
    TEXT.get_or_init(|| {
        let mut text = String::from(
            "pipes is an alias for pipelines.\nKinds: nushell, torch.\nLevels: intro, basic, intermediate, advanced.\nCategories:\n",
        );
        for category in NUSHELL_CATEGORIES.iter().chain(TORCH_CATEGORIES) {
            text.push_str(category);
            text.push('\n');
        }
        text
    })
    .as_str()
}

pub fn early_cli() -> Option<i32> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_none_or(|arg| arg != "tip") {
        return None;
    }
    Some(match dispatch(&args[1..]) {
        Ok(text) => {
            let text = color_early(&text);
            print!("{text}");
            let _ = std::io::Write::flush(&mut std::io::stdout());
            0
        }
        Err(reason) => {
            eprintln!("nutorch tip: {reason}");
            1
        }
    })
}

fn color_early(text: &str) -> String {
    if !text.starts_with("Tip · ") {
        return text.to_string();
    }
    let decision = color_decision_engine();
    let stack = Stack::new();
    if !coloring_enabled(&decision, &stack, std::io::stdout().is_terminal()) {
        return text.to_string();
    }
    let engine = shell_engine();
    highlight_rendered(&engine, &Stack::new(), text)
}

fn color_decision_engine() -> EngineState {
    let mut engine = EngineState::new();
    for name in ["FORCE_COLOR", "NO_COLOR", "CLICOLOR", "TERM"] {
        if let Ok(value) = std::env::var(name) {
            engine.add_env_var(name.to_string(), Value::string(value, Span::unknown()));
        }
    }
    engine
}

fn shell_engine() -> EngineState {
    let mut engine = nu_cmd_lang::add_default_context(EngineState::new());
    #[cfg(feature = "plugin")]
    {
        engine = nu_cmd_plugin::add_plugin_command_context(engine);
    }
    engine = nu_command::add_shell_command_context(engine);
    engine = nu_cmd_extra::add_extra_command_context(engine);
    engine = nu_cli::add_cli_context(engine);
    engine = nu_explore::add_explore_context(engine);
    crate::tensor::add_context(engine)
}

pub fn add_context(engine: &mut EngineState) {
    let mut working = StateWorkingSet::new(engine);
    working.add_decl(Box::new(TipCommand));
    engine
        .merge_delta(working.render())
        .expect("register nutorch tip");
}

pub fn violations(tips: &[Tip]) -> Vec<String> {
    let mut problems = Vec::new();
    if tips.len() < 300 {
        problems.push(format!(
            "catalog has {} tips; need at least 300",
            tips.len()
        ));
    }
    let torch = tips.iter().filter(|tip| tip.kind == "torch").count();
    if torch < 40 {
        problems.push(format!("catalog has {torch} torch tips; need at least 40"));
    }
    let intro = tips.iter().filter(|tip| tip.level == "intro").count();
    if intro < 40 {
        problems.push(format!("catalog has {intro} intro tips; need at least 40"));
    }
    let mut seen = std::collections::HashSet::new();
    for (index, tip) in tips.iter().enumerate() {
        let label = format!("{} {} {} #{index}", tip.kind, tip.r#type, tip.level);
        if !seen.insert(tip.body.clone()) {
            problems.push(format!("{label}: duplicate body"));
        }
        problems.extend(
            tip_problems(tip)
                .into_iter()
                .map(|problem| format!("{label}: {problem}")),
        );
    }
    for category in NUSHELL_CATEGORIES.iter().chain(TORCH_CATEGORIES) {
        let rows: Vec<&Tip> = tips.iter().filter(|tip| tip.r#type == *category).collect();
        if rows.len() < 4 {
            problems.push(format!("{category}: {} tips; need at least 4", rows.len()));
        }
        let levels = rows
            .iter()
            .map(|tip| tip.level.as_str())
            .collect::<std::collections::HashSet<_>>();
        if levels.len() < 2 {
            problems.push(format!("{category}: needs at least two levels"));
        }
        if !rows.iter().any(|tip| tip.level == "intro") {
            problems.push(format!("{category}: needs an intro tip"));
        }
        let expected_kind = if NUSHELL_CATEGORIES.contains(category) {
            "nushell"
        } else {
            "torch"
        };
        if rows.iter().any(|tip| tip.kind != expected_kind) {
            problems.push(format!("{category}: kind must be {expected_kind}"));
        }
    }
    for category in REQUIRED_ADVANCED {
        if !tips
            .iter()
            .any(|tip| tip.r#type == *category && tip.level == "advanced")
        {
            problems.push(format!("{category}: needs an advanced tip"));
        }
    }
    problems
}

fn tip_problems(tip: &Tip) -> Vec<String> {
    let mut problems = Vec::new();
    if tip.kind != "nushell" && tip.kind != "torch" {
        problems.push(format!("unknown kind {}", tip.kind));
    }
    if !LEVELS.contains(&tip.level.as_str()) {
        problems.push(format!("unknown level {}", tip.level));
    }
    let allowed = if tip.kind == "torch" {
        TORCH_CATEGORIES
    } else {
        NUSHELL_CATEGORIES
    };
    if !allowed.contains(&tip.r#type.as_str()) {
        problems.push(format!(
            "category {} is not a {} category",
            tip.r#type, tip.kind
        ));
    }
    if !tip.body.is_ascii() {
        problems.push("body is not ASCII".into());
    }
    let lines: Vec<&str> = tip.body.lines().collect();
    if !(1..=8).contains(&lines.len()) {
        problems.push(format!("body has {} lines; need 1 to 8", lines.len()));
    }
    if lines.first().is_none_or(|line| !line.starts_with("# ")) {
        problems.push("first line must be a # comment".into());
    }
    for line in &lines {
        if line.chars().count() > 72 {
            problems.push(format!("line longer than 72: {line}"));
        }
    }
    for phrase in DENY_PHRASES {
        if tip.body.contains(phrase) {
            problems.push(format!("body contains {phrase}"));
        }
    }
    for word in DENY_WORDS {
        if contains_word(&tip.body, word) {
            problems.push(format!("body contains {word}"));
        }
    }
    match classify(&tip.body) {
        Ok(level) if level != tip.level => {
            problems.push(format!("shape is {level}, declared {}", tip.level));
        }
        Err(reason) => problems.push(reason),
        Ok(_) => {}
    }
    match tip.kind.as_str() {
        "nushell" => {
            if contains_word(&tip.body, "torch") || contains_word(&tip.body, "nutorch") {
                problems.push("nushell tip contains torch or nutorch".into());
            }
        }
        "torch" => {
            let use_at = lines.iter().position(|line| *line == "use torch");
            match use_at {
                None => problems.push("torch tip needs a line that is exactly `use torch`".into()),
                Some(index) => {
                    if !lines[index + 1..]
                        .iter()
                        .any(|line| line.contains("torch "))
                    {
                        problems.push("torch tip needs a later torch command".into());
                    }
                }
            }
        }
        _ => {}
    }
    match tip.r#type.as_str() {
        "jobs" if !tip.body.contains("job spawn") => {
            problems.push("jobs tip must call job spawn".into());
        }
        "externs" if !tip.body.contains("^printf") && !tip.body.contains("^true") => {
            problems.push("externs tip must call ^printf or ^true".into());
        }
        "network"
            if !tip.body.contains("url parse")
                && !tip.body.contains("url join")
                && !tip.body.contains("url build") =>
        {
            problems.push("network tip must call url parse, url join, or url build".into());
        }
        "line-editor"
            if !tip.body.contains("$env.config.edit_mode")
                && !tip.body.contains("keybindings list") =>
        {
            problems.push("line-editor tip must read edit_mode or call keybindings list".into());
        }
        _ => {}
    }
    problems
}

fn classify(body: &str) -> Result<&'static str, String> {
    let stripped = strip_code(body);
    let multiline = is_multiline(&stripped);
    let has_def = has_keyword(&stripped, "def ") || has_keyword(&stripped, "export ");
    let (pipes, closures) = count_pipes(&stripped);
    if multiline {
        return Ok("advanced");
    }
    if has_def {
        return Err("same-line def or export".into());
    }
    if pipes >= 4 {
        return Ok("advanced");
    }
    if pipes == 3 || (closures > 0 && pipes < 4) {
        return Ok("intermediate");
    }
    if pipes == 2 && closures == 0 {
        return Ok("basic");
    }
    if pipes <= 1 && closures == 0 {
        return Ok("intro");
    }
    Err(format!("unclassified pipes={pipes} closures={closures}"))
}

fn strip_code(body: &str) -> String {
    let chars: Vec<char> = body.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '#' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '"' | '\'' => {
                let quote = chars[i];
                i += 1;
                while i < chars.len() {
                    if quote == '"' && chars[i] == '\\' {
                        i += 2;
                        continue;
                    }
                    if quote == '\'' && chars[i] == '\'' && chars.get(i + 1) == Some(&'\'') {
                        i += 2;
                        continue;
                    }
                    if chars[i] == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                out.push(' ');
            }
            other => {
                out.push(other);
                i += 1;
            }
        }
    }
    out
}

fn count_pipes(stripped: &str) -> (usize, usize) {
    let chars: Vec<char> = stripped.chars().collect();
    let mut pipes = 0;
    let mut closures = 0;
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && chars[j] == '|' {
                closures += 1;
                i = j + 1;
                while i < chars.len() && chars[i] != '|' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
                continue;
            }
        }
        if chars[i] == '|' {
            pipes += 1;
        }
        i += 1;
    }
    (pipes, closures)
}

fn is_multiline(stripped: &str) -> bool {
    let chars: Vec<char> = stripped.chars().collect();
    let keywords = ["export module ", "export def ", "def ", "module "];
    let mut i = 0;
    while i < chars.len() {
        if !word_boundary(&chars, i) {
            i += 1;
            continue;
        }
        let rest: String = chars[i..].iter().collect();
        let Some(keyword) = keywords.iter().find(|keyword| rest.starts_with(**keyword)) else {
            i += 1;
            continue;
        };
        let after = i + keyword.len();
        if let Some(rel) = chars[after..].iter().position(|c| *c == '{') {
            let brace = after + rel;
            let open_line = chars[..=brace].iter().filter(|c| **c == '\n').count();
            let mut depth = 0;
            for (k, ch) in chars.iter().enumerate().skip(brace) {
                if *ch == '{' {
                    depth += 1;
                }
                if *ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        let close_line = chars[..=k].iter().filter(|c| **c == '\n').count();
                        if close_line != open_line {
                            return true;
                        }
                        break;
                    }
                }
            }
        }
        i = after;
    }
    false
}

fn has_keyword(stripped: &str, keyword: &str) -> bool {
    let chars: Vec<char> = stripped.chars().collect();
    let keyword: Vec<char> = keyword.chars().collect();
    if chars.len() < keyword.len() {
        return false;
    }
    (0..=chars.len() - keyword.len()).any(|index| {
        word_boundary(&chars, index) && chars[index..index + keyword.len()] == keyword[..]
    })
}

fn word_boundary(chars: &[char], index: usize) -> bool {
    index == 0 || !chars[index - 1].is_ascii_alphanumeric()
}

fn contains_word(text: &str, word: &str) -> bool {
    let bytes = text.as_bytes();
    let word = word.as_bytes();
    if word.is_empty() || bytes.len() < word.len() {
        return false;
    }
    (0..=bytes.len() - word.len()).any(|index| {
        let before = index == 0 || !bytes[index - 1].is_ascii_alphanumeric();
        let after =
            index + word.len() == bytes.len() || !bytes[index + word.len()].is_ascii_alphanumeric();
        before && after && &bytes[index..index + word.len()] == word
    })
}

fn filter(
    kind: Option<&str>,
    category: Option<&str>,
    level: Option<&str>,
    echoed: &[String],
) -> Result<Vec<Tip>, String> {
    if let Some(kind) = kind
        && kind != "nushell"
        && kind != "torch"
    {
        return Err("unknown kind. Accepted kinds: nushell, torch.".into());
    }
    if let Some(level) = level
        && !LEVELS.contains(&level)
    {
        return Err("unknown level. Accepted levels: intro, basic, intermediate, advanced.".into());
    }
    let category = match category {
        None => None,
        Some("pipes") => Some("pipelines"),
        Some(category) if all_categories().any(|name| name == category) => Some(category),
        Some(_) => {
            return Err(format!(
                "unknown category. Accepted categories: {}.",
                all_categories().collect::<Vec<_>>().join(", ")
            ));
        }
    };
    let matches: Vec<Tip> = catalog()
        .iter()
        .filter(|tip| kind.is_none_or(|kind| tip.kind == kind))
        .filter(|tip| category.is_none_or(|category| tip.r#type == category))
        .filter(|tip| level.is_none_or(|level| tip.level == level))
        .cloned()
        .collect();
    if matches.is_empty() {
        let mut message = String::from("no tip matched");
        if !echoed.is_empty() {
            message.push(' ');
            message.push_str(&echoed.join(" "));
        }
        return Err(message);
    }
    Ok(matches)
}

fn all_categories() -> impl Iterator<Item = &'static str> {
    NUSHELL_CATEGORIES.iter().chain(TORCH_CATEGORIES).copied()
}

fn dispatch(args: &[std::ffi::OsString]) -> Result<String, String> {
    let mut kind = None;
    let mut category = None;
    let mut level = None;
    let mut help = false;
    let mut echoed = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].to_str().ok_or("argument is not UTF-8")?;
        if arg == "--help" || arg == "-h" {
            help = true;
            index += 1;
            continue;
        }
        if let Some(flag) = arg.strip_prefix("--") {
            let (name, inline) = match flag.split_once('=') {
                Some((name, value)) => (name, Some(value.to_string())),
                None => (flag, None),
            };
            if !matches!(name, "kind" | "type" | "level") {
                return Err(format!("unknown flag --{name}"));
            }
            let value = if let Some(value) = inline {
                value
            } else {
                index += 1;
                let next = args.get(index).ok_or(format!("--{name} expects a value"))?;
                let next = next.to_str().ok_or("argument is not UTF-8")?;
                if next.starts_with('-') {
                    return Err(format!("--{name} expects a value"));
                }
                next.to_string()
            };
            let slot = match name {
                "kind" => &mut kind,
                "type" => &mut category,
                "level" => &mut level,
                _ => unreachable!(),
            };
            if slot.is_some() {
                return Err(format!("duplicate flag --{name}"));
            }
            echoed.push(format!("--{name} {value}"));
            *slot = Some(value);
            index += 1;
            continue;
        }
        if arg.starts_with('-') {
            return Err(format!("unknown flag {arg}"));
        }
        return Err(format!("unexpected argument {arg}"));
    }
    if help {
        return Ok(help_text());
    }
    let matches = filter(
        kind.as_deref(),
        category.as_deref(),
        level.as_deref(),
        &echoed,
    )?;
    Ok(render(choose(&matches, random_index(matches.len()))))
}

fn random_index(len: usize) -> u64 {
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes).expect("getrandom");
    u64::from_le_bytes(bytes) % len as u64
}

#[derive(Clone)]
struct TipCommand;

impl Command for TipCommand {
    fn name(&self) -> &str {
        "nutorch tip"
    }
    fn description(&self) -> &str {
        "Print one usage tip from the built-in catalog."
    }
    fn extra_description(&self) -> &str {
        help_extra()
    }
    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Nothing, Type::Nothing)])
            .named(
                "kind",
                SyntaxShape::String,
                "Limit tips to nushell or torch.",
                None,
            )
            .named(
                "type",
                SyntaxShape::String,
                "Limit tips to one category. pipes is an alias for pipelines.",
                None,
            )
            .named(
                "level",
                SyntaxShape::String,
                "Limit tips to intro, basic, intermediate, or advanced.",
                None,
            )
            .category(Category::System)
    }
    fn run(
        &self,
        engine: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let kind: Option<String> = call.get_flag(engine, stack, "kind")?;
        let category: Option<String> = call.get_flag(engine, stack, "type")?;
        let level: Option<String> = call.get_flag(engine, stack, "level")?;
        match query(kind.as_deref(), category.as_deref(), level.as_deref()) {
            Ok(matches) => {
                let rendered = render(choose(&matches, random_index(matches.len())));
                print!(
                    "{}",
                    present(engine, stack, &rendered, std::io::stdout().is_terminal())
                );
                let _ = std::io::Write::flush(&mut std::io::stdout());
                Ok(PipelineData::empty())
            }
            Err(reason) => Err(ShellError::Generic(
                nu_protocol::shell_error::generic::GenericError::new(
                    "NuTorch tip failed",
                    reason,
                    call.head,
                ),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tip_level_shapes_match_the_rules() {
        assert_eq!(classify("[1 2 3] | math sum").unwrap(), "intro");
        assert_eq!(classify("{a: 1} | get a").unwrap(), "intro");
        assert_eq!(classify("module demo {}").unwrap(), "intro");
        assert_eq!(classify("[1 2 3] | math sum | math abs").unwrap(), "basic");
        assert_eq!(
            classify("[1 2 3] | where {|n| $n > 2} | math sum").unwrap(),
            "intermediate"
        );
        assert_eq!(
            classify("[1 2 3] | each {|n| $n | math abs} | math sum").unwrap(),
            "intermediate"
        );
        assert_eq!(
            classify("[1 2 3 4] | math sum | into string | str length | math abs").unwrap(),
            "advanced"
        );
        assert_eq!(
            classify("def greet [name: string] {\n  \"hi\"\n}\ngreet nu").unwrap(),
            "advanced"
        );
        assert!(classify("def greet [] { \"hi\" }").is_err());
        assert!(!contains_word("str starts-with", "start"));
        assert!(!contains_word("format", "rm"));
        assert!(contains_word("plugin-path", "plugin"));
    }

    #[test]
    fn tip_catalog_meets_the_floor_and_choose_reaches_each_tip() {
        let tips = catalog();
        assert!(
            violations(tips).is_empty(),
            "{}",
            violations(tips).join("\n")
        );
        let mut seen = std::collections::HashSet::new();
        for index in 0..tips.len() {
            seen.insert(choose(tips, index as u64).body.clone());
        }
        assert_eq!(seen.len(), tips.len());
        assert_eq!(choose(tips, tips.len() as u64).body, tips[0].body);
    }

    #[test]
    fn tip_filters_alias_pipes_and_reject_unknown_values() {
        let pipes = query(None, Some("pipes"), None).unwrap();
        assert!(pipes.iter().all(|tip| tip.r#type == "pipelines"));
        assert!(pipes.iter().all(|tip| tip.kind == "nushell"));
        let err = query(Some("nushell"), Some("tensors"), None).unwrap_err();
        assert_eq!(err, "no tip matched --kind nushell --type tensors");
        assert!(
            query(Some("nope"), None, None)
                .unwrap_err()
                .contains("nushell")
        );
        assert!(
            query(None, None, Some("nope"))
                .unwrap_err()
                .contains("advanced")
        );
        assert!(
            query(None, Some("nope"), None)
                .unwrap_err()
                .contains("pipelines")
        );
        let torch = query(Some("torch"), Some("tensors"), Some("intro")).unwrap();
        assert_eq!(torch.len(), 1);
        assert!(render(&torch[0]).starts_with("Tip · torch · tensors · intro\n"));
        assert!(!render(&torch[0]).contains(META_LINE));
    }

    #[test]
    fn tip_early_flags_report_duplicates_and_help() {
        let help = dispatch(&["--help".into()]).unwrap();
        assert!(help.contains("pipes is an alias for pipelines."));
        for category in NUSHELL_CATEGORIES.iter().chain(TORCH_CATEGORIES) {
            assert!(help.contains(category), "{category} missing from help");
        }
        assert!(help.contains("intro") && help.contains("advanced"));
        assert!(
            dispatch(&[
                "--kind".into(),
                "nushell".into(),
                "--kind".into(),
                "torch".into()
            ])
            .unwrap_err()
            .contains("duplicate flag --kind")
        );
        assert!(
            dispatch(&["--type".into()])
                .unwrap_err()
                .contains("--type expects a value")
        );
        assert!(
            dispatch(&["--nope".into()])
                .unwrap_err()
                .contains("unknown flag --nope")
        );
        assert!(
            dispatch(&["extra".into()])
                .unwrap_err()
                .contains("unexpected argument extra")
        );
        let err = dispatch(&[
            "--type".into(),
            "pipes".into(),
            "--kind".into(),
            "torch".into(),
        ])
        .unwrap_err();
        assert!(
            err.contains("no tip matched --type pipes --kind torch"),
            "{err}"
        );
    }

    #[test]
    fn tip_bodies_run_on_nu_and_nutorch() {
        let nu = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../forks/nushell/target/debug/nu");
        assert!(nu.is_file(), "pinned nu is missing at {}", nu.display());
        let home = tempfile::tempdir().unwrap();
        let mut failures = Vec::new();
        for tip in catalog() {
            if tip.kind != "nushell" {
                continue;
            }
            let output = std::process::Command::new(&nu)
                .arg("--no-config-file")
                .arg("-c")
                .arg(&tip.body)
                .env("HOME", home.path())
                .env("XDG_CONFIG_HOME", home.path())
                .output()
                .unwrap();
            if !output.status.success() {
                failures.push(format!(
                    "nu {} {}: {}\n{}",
                    tip.r#type,
                    tip.level,
                    tip.body,
                    String::from_utf8_lossy(&output.stderr)
                ));
                if failures.len() == 20 {
                    break;
                }
            }
        }
        let base = test_engine();
        if failures.len() < 20 {
            let dir = home.path().to_string_lossy().to_string();
            let path = Value::list(
                vec![Value::test_string("/usr/bin"), Value::test_string("/bin")],
                Span::unknown(),
            );
            for tip in catalog() {
                let mut engine = base.clone();
                engine.add_env_var("PWD".into(), Value::test_string(&dir));
                engine.add_env_var("HOME".into(), Value::test_string(&dir));
                engine.add_env_var("PATH".into(), path.clone());
                let mut stack = Stack::new();
                stack.add_env_var("PWD".into(), Value::test_string(&dir));
                stack.add_env_var("HOME".into(), Value::test_string(&dir));
                stack.add_env_var("PATH".into(), path.clone());
                let code = nu_cli::eval_source(
                    &mut engine,
                    &mut stack,
                    tip.body.as_bytes(),
                    "tip",
                    PipelineData::empty(),
                    false,
                );
                if code != 0 {
                    failures.push(format!(
                        "engine {} {} {} exit {code}\n{}",
                        tip.kind, tip.r#type, tip.level, tip.body
                    ));
                    if failures.len() == 20 {
                        break;
                    }
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
    }

    fn test_engine() -> EngineState {
        let mut engine = super::shell_engine();
        engine.add_env_var(
            "config".into(),
            nu_protocol::Config::default().into_value(Span::unknown()),
        );
        engine.generate_nu_constant();
        engine
    }

    fn set_color(engine: &mut EngineState, mode: nu_protocol::UseAnsiColoring) {
        let mut config = (*engine.config).clone();
        config.use_ansi_coloring = mode;
        engine.config = std::sync::Arc::new(config);
    }

    fn styled(engine: &EngineState, line: &str) -> reedline::StyledText {
        let highlighter = nu_cli::NuHighlighter::new(
            std::sync::Arc::new(engine.clone()),
            std::sync::Arc::new(Stack::new()),
        );
        reedline::Highlighter::highlight(&highlighter, line, 0)
    }

    fn strip_ansi(text: &str) -> String {
        let mut out = String::new();
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' && chars.peek() == Some(&'[') {
                chars.next();
                for next in chars.by_ref() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
            out.push(ch);
        }
        out
    }

    fn sgr_sequences(text: &str) -> std::collections::BTreeSet<String> {
        let mut found = std::collections::BTreeSet::new();
        let bytes = text.as_bytes();
        let mut index = 0;
        while index + 1 < bytes.len() {
            if bytes[index] == 0x1b && bytes[index + 1] == b'[' {
                let start = index;
                index += 2;
                while index < bytes.len() && !bytes[index].is_ascii_alphabetic() {
                    index += 1;
                }
                if index < bytes.len() {
                    index += 1;
                }
                found.insert(String::from_utf8_lossy(&bytes[start..index]).into_owned());
            } else {
                index += 1;
            }
        }
        found
    }

    #[test]
    fn tip_highlight_uses_distinct_styles_and_strips_clean() {
        let engine = test_engine();
        let stack = Stack::new();
        let pipeline = "[1 2 3] | math sum";
        let lesson = "# add the list";
        let painted = highlight_body(&engine, &stack, pipeline);
        assert!(painted.contains('\u{1b}'), "{painted:?}");
        assert!(sgr_sequences(&painted).len() >= 2, "{painted:?}");
        assert_eq!(strip_ansi(&painted), pipeline);
        let comment = highlight_body(&engine, &stack, lesson);
        assert_eq!(strip_ansi(&comment), lesson);
        let pipeline_styled = styled(&engine, pipeline);
        let lesson_styled = styled(&engine, lesson);
        assert!(!pipeline_styled.buffer.is_empty());
        assert!(!lesson_styled.buffer.is_empty());
        assert_ne!(
            pipeline_styled.buffer[0].0, lesson_styled.buffer[0].0,
            "pipeline {:?} lesson {:?}",
            pipeline_styled.buffer[0].0, lesson_styled.buffer[0].0
        );
    }

    #[test]
    fn tip_color_follows_config_and_no_color() {
        let engine = test_engine();
        let stack = Stack::new();
        let tip = Tip {
            kind: "nushell".into(),
            r#type: "math".into(),
            level: "intro".into(),
            body: "# add the list\n[1 2 3] | math sum\n".into(),
        };
        let rendered = render(&tip);

        let mut off = engine.clone();
        set_color(&mut off, nu_protocol::UseAnsiColoring::False);
        off.add_env_var("FORCE_COLOR".into(), Value::test_string("1"));
        let plain = present(&off, &stack, &rendered, true);
        assert!(!plain.contains('\u{1b}'), "{plain:?}");
        assert!(plain.contains("[1 2 3] | math sum"));

        let mut on = engine.clone();
        set_color(&mut on, nu_protocol::UseAnsiColoring::True);
        on.add_env_var("NO_COLOR".into(), Value::test_string("1"));
        let colored = present(&on, &stack, &rendered, false);
        let mut lines = colored.lines();
        let label = lines.next().unwrap();
        assert!(!label.contains('\u{1b}'), "{label:?}");
        assert!(colored.contains('\u{1b}'), "{colored:?}");
        assert_eq!(strip_ansi(&colored), rendered);

        let mut auto = engine.clone();
        set_color(&mut auto, nu_protocol::UseAnsiColoring::Auto);
        auto.add_env_var("NO_COLOR".into(), Value::test_string("1"));
        let muted = present(&auto, &stack, &rendered, true);
        assert!(!muted.contains('\u{1b}'), "{muted:?}");
        assert_eq!(muted, rendered);
    }
}
