use crate::{
    command,
    config_files::{self, setup_config},
};
use log::trace;
#[cfg(feature = "plugin")]
use nu_cli::read_plugin_file;
use nu_cli::{EvaluateCommandsOpts, evaluate_commands, evaluate_file, evaluate_repl};
use nu_config::ConfigFileKind;
use nu_protocol::{
    BannerKind, PipelineData, ShellError, Spanned,
    engine::{EngineState, Stack},
    report_shell_error,
};
use nu_utils::perf;
use nu_utils::time::Instant;

pub(crate) fn run_commands(
    engine_state: &mut EngineState,
    mut stack: Stack,
    parsed_nu_cli_args: command::NushellCliArgs,
    use_color: bool,
    commands: &Spanned<String>,
    input: PipelineData,
    args_to_script: Vec<String>,
    entire_start_time: nu_utils::time::Instant,
) {
    trace!("run_commands");

    let start_time = nu_utils::time::Instant::now();
    let create_scaffold = !engine_state.config_dirs.config_home.exists();

    if parsed_nu_cli_args.no_config_file.is_none() {
        #[cfg(feature = "plugin")]
        read_plugin_file(
            engine_state,
            parsed_nu_cli_args.plugin_file.as_ref().map(|s| s.span),
        );

        perf!("read plugins", start_time, use_color);

        let start_time = Instant::now();
        if engine_state.config_dirs.env_file.is_override()
            || parsed_nu_cli_args.login_shell.is_some()
        {
            config_files::read_config_file(
                engine_state,
                &mut stack,
                ConfigFileKind::Env,
                create_scaffold,
                true,
                parsed_nu_cli_args.env_file.as_ref(),
            );
        } else {
            config_files::read_default_env_file(engine_state, &mut stack)
        }

        perf!("read env.nu", start_time, use_color);

        let start_time = Instant::now();

        if engine_state.config_dirs.config_file.is_override()
            || parsed_nu_cli_args.login_shell.is_some()
        {
            config_files::read_config_file(
                engine_state,
                &mut stack,
                ConfigFileKind::Config,
                create_scaffold,
                true,
                parsed_nu_cli_args.config_file.as_ref(),
            );
        }

        perf!("read config.nu", start_time, use_color);

        let start_time = Instant::now();
        if parsed_nu_cli_args.login_shell.is_some() {
            config_files::read_loginshell_file(engine_state, &mut stack, false);
        }

        perf!("read login.nu", start_time, use_color);
    }

    engine_state.set_startup_time(entire_start_time.elapsed().as_nanos() as i64);
    engine_state.generate_nu_constant();

    let start_time = Instant::now();
    let result = evaluate_commands(
        commands,
        engine_state,
        &mut stack,
        input,
        args_to_script,
        EvaluateCommandsOpts {
            table_mode: parsed_nu_cli_args.table_mode,
            error_style: parsed_nu_cli_args.error_style,
            no_newline: parsed_nu_cli_args.no_newline.is_some(),
        },
    );
    perf!("evaluate_commands", start_time, use_color);

    if let Err(err) = result {
        // Match upstream nu: Exit must process::exit(code) without report_shell_error
        // (exit_code() maps Exit to 1 and the report message confuses users).
        if let ShellError::Exit { code, .. } = &err {
            std::process::exit(*code)
        }
        report_shell_error(Some(&stack), engine_state, &err);
        std::process::exit(err.exit_code().unwrap_or(0));
    }
}

pub(crate) fn run_file(
    engine_state: &mut EngineState,
    mut stack: Stack,
    parsed_nu_cli_args: command::NushellCliArgs,
    use_color: bool,
    script_name: String,
    args_to_script: Vec<String>,
    input: PipelineData,
) {
    trace!("run_file");

    if parsed_nu_cli_args.no_config_file.is_none() {
        let start_time = Instant::now();
        let create_scaffold = !engine_state.config_dirs.config_home.exists();
        #[cfg(feature = "plugin")]
        read_plugin_file(
            engine_state,
            parsed_nu_cli_args.plugin_file.as_ref().map(|s| s.span),
        );
        perf!("read plugins", start_time, use_color);

        let start_time = Instant::now();
        if engine_state.config_dirs.env_file.is_override() {
            config_files::read_config_file(
                engine_state,
                &mut stack,
                ConfigFileKind::Env,
                create_scaffold,
                true,
                parsed_nu_cli_args.env_file.as_ref(),
            );
        } else {
            config_files::read_default_env_file(engine_state, &mut stack)
        }
        perf!("read env.nu", start_time, use_color);

        let start_time = Instant::now();
        if engine_state.config_dirs.config_file.is_override() {
            config_files::read_config_file(
                engine_state,
                &mut stack,
                ConfigFileKind::Config,
                create_scaffold,
                true,
                parsed_nu_cli_args.config_file.as_ref(),
            );
        }
        perf!("read config.nu", start_time, use_color);
    }

    engine_state.generate_nu_constant();

    let start_time = Instant::now();
    let result = evaluate_file(
        script_name,
        &args_to_script,
        engine_state,
        &mut stack,
        input,
    );
    perf!("evaluate_file", start_time, use_color);

    if let Err(err) = result {
        if let ShellError::Exit { code, .. } = &err {
            std::process::exit(*code)
        }
        report_shell_error(Some(&stack), engine_state, &err);
        std::process::exit(err.exit_code().unwrap_or(0));
    }
}

/// Interactive intro lines for Nushell's full and short banner settings.
/// `BannerKind::None` prints nothing. A trailing empty string is the blank
/// line after the banner.
fn intro_banner_lines(
    kind: BannerKind,
    version: &str,
    nu_version: &str,
    startup: std::time::Duration,
) -> Vec<String> {
    let green = "\x1b[32m";
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";
    let fg = "\x1b[37m";
    let startup_line = format!("{green}{bold}Startup Time:{reset}{fg} {startup:?}{reset}");
    match kind {
        BannerKind::None => Vec::new(),
        BannerKind::Short => vec![startup_line, String::new()],
        BannerKind::Full => vec![
            format!(
                "{fg}Welcome to {green}{bold}NuTorch{reset}{fg}, based on the {green}nu{reset}{fg} language{reset}"
            ),
            format!("{fg}Version: {green}{version}{fg} (nushell {green}{nu_version}{fg}){reset}"),
            startup_line,
            String::new(),
        ],
    }
}

pub(crate) fn run_repl(
    engine_state: &mut EngineState,
    mut stack: Stack,
    parsed_nu_cli_args: command::NushellCliArgs,
    entire_start_time: nu_utils::time::Instant,
    sync_context: std::sync::Arc<nutorch::sync::Context>,
) -> Result<(), miette::ErrReport> {
    trace!("run_repl");
    let start_time = nu_utils::time::Instant::now();

    if parsed_nu_cli_args.no_config_file.is_none() {
        setup_config(
            engine_state,
            &mut stack,
            parsed_nu_cli_args.login_shell.is_some(),
        );
    }

    let use_color = engine_state
        .get_config()
        .use_ansi_coloring
        .get(engine_state);
    perf!("setup_config", start_time, use_color);

    stack.add_env_var(
        "NUTORCH_MODE".to_string(),
        nu_protocol::Value::string("nu", nu_protocol::Span::unknown()),
    );

    nu_cli::eval_source(
        engine_state,
        &mut stack,
        br#"$env.PROMPT_COMMAND = {||
            let mode = ($env.NUTORCH_MODE? | default "nu")
            let color = match $mode {
                "nu" => (ansi green)
                "ai" => (ansi purple)
                _ => (ansi green)
            }
            let reset = (ansi reset)
            let dir = if ($env.PWD | str starts-with $env.HOME) {
                $env.PWD | str replace $env.HOME "~"
            } else {
                $env.PWD
            }
            $"($color)[($mode)](ansi reset) ($dir)"
        }"#,
        "nutorch-prompt",
        nu_protocol::PipelineData::empty(),
        false,
    );

    {
        let show_banner = engine_state.get_config().show_banner.clone();
        nu_cli::eval_source(
            engine_state,
            &mut stack,
            b"$env.config.show_banner = false",
            "nutorch-banner-disable",
            nu_protocol::PipelineData::empty(),
            false,
        );
        let version = env!("CARGO_PKG_VERSION");
        let nu_version = env!("NUSHELL_VERSION");
        for line in intro_banner_lines(
            show_banner,
            version,
            nu_version,
            entire_start_time.elapsed(),
        ) {
            eprintln!("{line}");
        }
    }

    let server = sync_context.start(engine_state, &mut stack);
    let dispatcher = nutorch::sync::Dispatcher {
        server: server.clone(),
    };
    let dispatcher: std::sync::Arc<std::sync::Mutex<Box<dyn nu_cli::ModeDispatcher>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Box::new(dispatcher)));

    let start_time = nu_utils::time::Instant::now();
    let ret_val = evaluate_repl(
        engine_state,
        stack,
        parsed_nu_cli_args.execute,
        parsed_nu_cli_args.no_std_lib,
        entire_start_time,
        Some(dispatcher),
    );
    perf!("evaluate_repl", start_time, use_color);
    if let Some(server) = server {
        server.shutdown();
    }

    ret_val
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const REMOVED_HINTS: &[&str] = &[
        "Press Shift+Tab to toggle between Nushell and AI (unfinished).",
        "The AI mode is unfinished; input is never executed.",
        "Type roamari to browse the web.",
        "Type ah then Tab to see more commands.",
        "Shift+Tab: Nushell ↔ AI (unfinished)",
    ];

    #[test]
    fn intro_banner_omits_hint_lines() {
        let startup = Duration::from_millis(12);
        let full = intro_banner_lines(BannerKind::Full, "2.0.9", "0.115.2", startup);
        let short = intro_banner_lines(BannerKind::Short, "2.0.9", "0.115.2", startup);
        let full_text = full.join("\n");
        let short_text = short.join("\n");

        for hint in REMOVED_HINTS {
            assert!(
                !full_text.contains(hint),
                "{hint} still in full banner: {full_text:?}"
            );
            assert!(
                !short_text.contains(hint),
                "{hint} still in short banner: {short_text:?}"
            );
        }

        let welcome = full_text.find("Welcome to").expect("welcome");
        let version = full_text.find("Version:").expect("version");
        let startup_at = full_text.find("Startup Time:").expect("startup");
        assert!(welcome < version && version < startup_at);
        assert!(full_text.contains("\u{1b}[32m\u{1b}[1mNuTorch"));
        assert!(full_text.contains("2.0.9"));
        assert!(full_text.contains("nushell \u{1b}[32m0.115.2"));
        assert!(full_text.contains(&format!("{startup:?}")));
        assert!(full_text.ends_with('\n'));

        assert_eq!(short.len(), 2);
        assert!(short[0].contains("Startup Time:"));
        assert!(short[0].contains(&format!("{startup:?}")));
        assert!(short[1].is_empty());
        assert!(!short_text.contains("Welcome to"));
        assert!(!short_text.contains("Version:"));

        assert!(intro_banner_lines(BannerKind::None, "2.0.9", "0.115.2", startup).is_empty());
    }
}
