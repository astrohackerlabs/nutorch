//! `nutorch tip` from the shipped binary: early CLI, help, and in-process command.

use std::process::Command;

mod support;

const CATEGORIES: &[&str] = &[
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

fn tip(args: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_nutorch"));
    command.args(args);
    command
        .env_remove("NUTORCH_SYNC_SOCKET")
        .env_remove("NUTORCH_SYNC_TOKEN");
    command.output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[test]
fn tip_prints_one_catalog_entry() {
    let output = tip(&["tip"]);
    assert!(output.status.success(), "{}", text(&output.stderr));
    let stdout = text(&output.stdout);
    assert!(!stdout.contains("Type nutorch tip for more tips."));
    let mut lines = stdout.lines();
    let label = lines.next().unwrap();
    assert!(label.starts_with("Tip · "), "{label}");
    assert!(!stdout.contains('\u{1b}'), "{stdout}");
    let body: Vec<&str> = lines.collect();
    assert!(!body.is_empty() && body.len() <= 8, "{stdout}");
    assert!(
        body.iter().all(|line| line.chars().count() <= 72),
        "{stdout}"
    );
}

#[test]
fn tip_type_pipes_prints_a_pipelines_label() {
    let stdout = text(&tip(&["tip", "--type", "pipes"]).stdout);
    assert!(
        stdout
            .lines()
            .next()
            .unwrap()
            .contains("nushell · pipelines"),
        "{stdout}"
    );
}

#[test]
fn tip_torch_tensors_intro_label_is_exact() {
    let output = tip(&[
        "tip", "--kind", "torch", "--type", "tensors", "--level", "intro",
    ]);
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stdout).lines().next().unwrap(),
        "Tip · torch · tensors · intro"
    );
}

#[test]
fn tip_empty_filter_names_the_request() {
    let output = tip(&["tip", "--kind", "nushell", "--type", "tensors"]);
    assert!(!output.status.success());
    let stderr = text(&output.stderr);
    assert!(stderr.contains("no tip matched"), "{stderr}");
    assert!(stderr.contains("--kind nushell"), "{stderr}");
    assert!(stderr.contains("--type tensors"), "{stderr}");
}

#[test]
fn tip_unknown_values_name_the_accepted_set() {
    let kind = text(&tip(&["tip", "--kind", "nope"]).stderr);
    assert!(kind.contains("nushell") && kind.contains("torch"), "{kind}");
    let level = text(&tip(&["tip", "--level", "nope"]).stderr);
    assert!(
        level.contains("intro") && level.contains("advanced") && level.contains("intermediate"),
        "{level}"
    );
    let category = text(&tip(&["tip", "--type", "nope"]).stderr);
    assert!(category.contains("unknown category"), "{category}");
    for name in CATEGORIES {
        assert!(category.contains(name), "{name} missing from {category}");
    }
}

#[test]
fn tip_help_lists_kinds_levels_and_categories() {
    let output = tip(&["tip", "--help"]);
    assert!(output.status.success(), "{}", text(&output.stderr));
    let stdout = text(&output.stdout);
    for token in [
        "pipes",
        "pipelines",
        "nushell",
        "torch",
        "intro",
        "advanced",
    ] {
        assert!(stdout.contains(token), "{token} missing from help");
    }
    for name in CATEGORIES {
        assert!(stdout.contains(name), "{name} missing from help");
    }
    let short = tip(&["tip", "-h"]);
    assert!(short.status.success());
    assert!(text(&short.stdout).contains("pipes is an alias for pipelines."));
}

#[test]
fn tip_rejects_duplicate_missing_and_unknown_flags() {
    assert!(
        text(&tip(&["tip", "--kind", "nushell", "--kind", "torch"]).stderr)
            .contains("duplicate flag --kind")
    );
    assert!(text(&tip(&["tip", "--type"]).stderr).contains("--type expects a value"));
    assert!(text(&tip(&["tip", "--nope"]).stderr).contains("unknown flag --nope"));
    assert!(text(&tip(&["tip", "extra"]).stderr).contains("unexpected argument extra"));
    let help = tip(&["tip", "--help", "--type", "pipes"]);
    assert!(help.status.success(), "{}", text(&help.stderr));
    assert!(text(&help.stdout).contains("Usage:"));
    let missing = tip(&["tip", "--type", "--help"]);
    assert!(!missing.status.success());
    assert!(text(&missing.stderr).contains("--type expects a value"));
}

#[test]
fn tip_ignores_a_config_file_that_exits() {
    let home = tempfile::tempdir().unwrap();
    let config = home.path().join("nushell");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("config.nu"), "exit 7\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nutorch"))
        .args(["tip", "--type", "pipes"])
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path())
        .env_remove("NUTORCH_SYNC_SOCKET")
        .env_remove("NUTORCH_SYNC_TOKEN")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", text(&output.stderr));
    let stdout = text(&output.stdout);
    assert!(stdout.contains("nushell · pipelines"), "{stdout}");
    assert!(!stdout.contains("Welcome to"), "{stdout}");
    assert!(!stdout.contains('\u{1b}'), "{stdout}");
}

#[test]
fn tip_command_runs_inside_the_process() {
    let home = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nutorch"))
        .args(["--no-config-file", "-c", "nutorch tip --type pipes"])
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path())
        .env_remove("NUTORCH_SYNC_SOCKET")
        .env_remove("NUTORCH_SYNC_TOKEN")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        text(&output.stdout),
        text(&output.stderr)
    );
    let stdout = text(&output.stdout);
    assert!(stdout.contains("nushell · pipelines"), "{stdout}");
    assert!(!stdout.contains('\u{1b}'), "{stdout}");
}

#[cfg(unix)]
#[test]
fn tip_torch_tensors_intro_is_highlighted_on_a_pty() {
    let output = tip_on_pty(&[
        "tip", "--kind", "torch", "--type", "tensors", "--level", "intro",
    ]);
    assert!(!output.contains("Welcome to"), "{output}");
    assert!(!output.contains("tensor<shape="), "{output}");
    let label = "Tip · torch · tensors · intro";
    let marker = output.find("tensors").unwrap_or_else(|| panic!("{output}"));
    let label_start = output[..marker]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    let label_end = output[marker..]
        .find('\n')
        .map(|index| marker + index)
        .unwrap_or(output.len());
    let label_line = output[label_start..label_end].trim_end_matches('\r');
    assert!(label_line.contains('\u{1b}'), "{output}");
    assert_eq!(strip_ansi(label_line), label, "{output}");
    let body = &output[label_end..];
    assert!(body.contains('\u{1b}'), "{output}");
    let source = strip_ansi(body);
    assert!(source.contains("torch tensor"), "{source}");
    assert!(source.contains("use torch"), "{source}");
}

#[cfg(unix)]
#[test]
fn tip_loss_scope_is_highlighted_on_both_cli_paths() {
    let config = nu_protocol::Config::default();
    let variable = nu_color_config::get_shape_color("shape_variable", &config);
    let command = nu_color_config::get_shape_color("shape_internalcall", &config);
    let tip = nutorch::tips::catalog()
        .iter()
        .find(|tip| tip.kind == "torch" && tip.r#type == "loss" && tip.level == "intro")
        .unwrap();
    for args in [
        vec![
            "tip", "--kind", "torch", "--type", "loss", "--level", "intro",
        ],
        vec![
            "--no-config-file",
            "-c",
            "nutorch tip --kind torch --type loss --level intro",
        ],
    ] {
        let output = tip_on_pty(&args).replace("\r\n", "\n");
        assert_eq!(strip_ansi(&output), nutorch::tips::render(tip));
        for name in ["$pred", "$target"] {
            assert!(
                output.contains(&variable.paint(name).to_string()),
                "{output:?}"
            );
        }
        assert!(
            output.contains(&command.paint("torch mse_loss").to_string()),
            "{output:?}"
        );
    }
}

#[cfg(unix)]
fn tip_on_pty(args: &[&str]) -> String {
    use std::fs::File;
    use std::os::fd::AsRawFd;
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let home = tempfile::tempdir().unwrap();
    let size = nix::pty::Winsize {
        ws_row: 40,
        ws_col: 160,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let pty = support::open_pty(Some(&size));
    let slave = File::from(pty.slave);
    let mut command = Command::new(env!("CARGO_BIN_EXE_nutorch"));
    command
        .args(args)
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path())
        .env_remove("NUTORCH_SYNC_SOCKET")
        .env_remove("NUTORCH_SYNC_TOKEN")
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CLICOLOR")
        .env("TERM", "xterm-256color")
        .stdin(Stdio::null())
        .stdout(Stdio::from(slave.try_clone().unwrap()))
        .stderr(Stdio::from(slave));
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut master = File::from(pty.master);
    unsafe {
        libc::fcntl(master.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK);
    }
    let mut child = command.spawn().unwrap();
    let mut output = String::new();
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        read_pty(&mut master, &mut output);
        if let Some(status) = child.try_wait().unwrap() {
            for _ in 0..5 {
                let before = output.len();
                read_pty(&mut master, &mut output);
                if output.len() == before {
                    break;
                }
            }
            break status;
        }
        assert!(Instant::now() < deadline, "{output}");
    };
    assert!(status.success(), "{output}");
    output
}

#[cfg(unix)]
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

#[cfg(unix)]
fn read_pty(master: &mut std::fs::File, output: &mut String) {
    use std::io::Read;
    use std::os::fd::AsRawFd;

    let mut poll = libc::pollfd {
        fd: master.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    if unsafe { libc::poll(&mut poll, 1, 50) } <= 0 {
        return;
    }
    let mut bytes = [0u8; 8192];
    loop {
        match master.read(&mut bytes) {
            Ok(0) => return,
            Ok(count) => output.push_str(&String::from_utf8_lossy(&bytes[..count])),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return,
            Err(error) if error.raw_os_error() == Some(libc::EIO) => return,
            Err(error) => panic!("pty read failed: {error}; {output}"),
        }
    }
}

#[test]
fn nutorch_help_lists_tip_beside_sync() {
    let output = Command::new(env!("CARGO_BIN_EXE_nutorch"))
        .arg("--help")
        .env_remove("NUTORCH_SYNC_SOCKET")
        .env_remove("NUTORCH_SYNC_TOKEN")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", text(&output.stderr));
    let stdout = text(&output.stdout);
    assert!(stdout.contains("tip"), "{stdout}");
    assert!(stdout.contains("sync"), "{stdout}");
}
