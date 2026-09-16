use nutorch::sync::transport::{SOCKET, Server, TOKEN};
use std::{
    fs,
    os::unix::{ffi::OsStringExt, fs::PermissionsExt},
    process::Command,
};

fn home() -> tempfile::TempDir {
    let home = tempfile::Builder::new()
        .prefix("ntcli-")
        .tempdir_in("/tmp")
        .unwrap();
    fs::set_permissions(home.path(), fs::Permissions::from_mode(0o700)).unwrap();
    home
}
fn command(home: &std::path::Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nutorch"));
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home)
        .env("XDG_RUNTIME_DIR", home)
        .env_remove(SOCKET)
        .env_remove(TOKEN);
    cmd
}

#[test]
fn early_dispatch_help_errors_and_no_configuration() {
    let home = home();
    let config = home.path().join("nushell");
    fs::create_dir(&config).unwrap();
    fs::write(
        config.join("config.nu"),
        "error make {msg: 'CONFIG_MUST_NOT_RUN'}",
    )
    .unwrap();
    let mut top_help = None;
    for flag in ["--help", "-h"] {
        let out = command(home.path()).arg(flag).output().unwrap();
        assert!(out.status.success());
        assert!(out.stderr.is_empty());
        let help = String::from_utf8_lossy(&out.stdout);
        if let Some(previous) = &top_help {
            assert_eq!(previous, &out.stdout);
        } else {
            top_help = Some(out.stdout.clone());
        }
        let plain = nu_utils::strip_ansi_likely(&help);
        assert!(!plain.split_whitespace().any(|word| word == "nu"));
        for identifier in [
            "Nushell",
            "NU_LIB_DIRS",
            "config.nu",
            "nu_plugin_one",
            "nu_repl",
        ] {
            assert!(plain.contains(identifier));
        }
        for line in plain
            .lines()
            .filter(|line| line.trim_start().starts_with("Example:"))
        {
            assert!(line.trim_start().starts_with("Example: nutorch "));
        }
        for expected in [
            "nutorch [options] [script file] [script args]",
            "  nutorch sync\n",
            "Queue exported environment variables in the enclosing NuTorch.",
            "Updates apply at its next prompt. Missing keys are not deleted.",
            "Run nutorch sync --help for details.",
            "--interactive",
            "--config",
        ] {
            assert!(help.contains(expected), "missing {expected:?} in {flag}");
        }
        assert!(help.find("Commands:").unwrap() < help.find("Options:").unwrap());
        assert!(!home.path().join("astrohacker").exists());
    }
    let mut sync_help = None;
    for args in [vec!["sync", "--help"], vec!["sync", "-h"]] {
        let out = command(home.path()).args(args).output().unwrap();
        assert!(out.status.success());
        if let Some(previous) = &sync_help {
            assert_eq!(previous, &out.stdout);
        } else {
            sync_help = Some(out.stdout.clone());
        }
        let help = String::from_utf8_lossy(&out.stdout);
        let plain = nu_utils::strip_ansi_likely(&help);
        assert!(plain.contains("Usage:\n  nutorch sync\n"));
        assert!(plain.contains("Success means queued; updates apply at its next prompt."));
        assert!(plain.contains("Missing keys are not deleted."));
        assert!(plain.contains("Options:\n\nGeneral:\n  -h, --help\n      show this help message\n      Example: nutorch sync --help"));
        let top = String::from_utf8_lossy(top_help.as_ref().unwrap());
        assert_eq!(help.lines().next(), top.lines().next());
        // Compare actual styled lines, not just ANSI-stripped headings.
        for marker in ["General:", "--help", "show this help message"] {
            let line = help.lines().find(|line| line.contains(marker)).unwrap();
            assert!(top.lines().any(|candidate| candidate == line));
        }
        assert!(out.stderr.is_empty());
    }
    let bins = command(home.path())
        .args(["--testbin", "--help"])
        .output()
        .unwrap();
    // The existing parser rejects --help as a test-bin value before dispatch.
    assert_eq!(bins.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&bins.stderr).contains("Valid test bin values"));
    let invalid = command(home.path())
        .arg("--zzzz-invalid-option")
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    let error = String::from_utf8_lossy(&invalid.stderr);
    assert!(error.contains("nutorch --help"));
    assert!(!error.contains("CONFIG_MUST_NOT_RUN"));
    for args in [
        vec!["sync"],
        vec!["sync", "--bad"],
        vec!["--no-config-file", "-c", "nutorch sync"],
    ] {
        let out = command(home.path()).args(args).output().unwrap();
        assert!(!out.status.success());
        assert!(!String::from_utf8_lossy(&out.stderr).contains("CONFIG_MUST_NOT_RUN"));
    }
    assert!(!home.path().join("astrohacker").exists());
}

#[test]
fn external_and_noninteractive_native_target_inherited_receiver() {
    let home = home();
    let server = Server::start(Some(home.path())).unwrap();
    for args in [
        vec!["sync"],
        vec![
            "--no-config-file",
            "-c",
            "$env.CLI_SYNC = 'native'; nutorch sync",
        ],
        vec!["--no-config-file", "-c", "use torch nutorch; nutorch sync"],
    ] {
        let out = command(home.path())
            .env(SOCKET, &server.endpoint.path)
            .env(TOKEN, &server.endpoint.token)
            .env("CLI_SYNC", "external\n☃=literal;$value")
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let output = String::from_utf8_lossy(&out.stdout);
        assert!(output.contains("Queued environment snapshot"));
        assert!(!output.contains(&server.endpoint.token));
        assert!(!output.contains("literal;$value"));
        let pending = server.drain();
        assert_eq!(pending.len(), 1);
        assert!(pending[0].env.iter().any(|(k, _)| k == "CLI_SYNC"));
        assert!(
            !pending[0]
                .env
                .iter()
                .any(|(k, _)| k.starts_with("NUTORCH_SYNC_"))
        );
    }
    // A non-UTF-8 value must never silently become replacement characters.
    let out = command(home.path())
        .env(SOCKET, &server.endpoint.path)
        .env(TOKEN, &server.endpoint.token)
        .env("BAD_UTF8", std::ffi::OsString::from_vec(vec![0xff]))
        .arg("sync")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(server.drain().is_empty());
    server.shutdown();
    let out = command(home.path())
        .env(SOCKET, &server.endpoint.path)
        .env(TOKEN, &server.endpoint.token)
        .arg("sync")
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn malformed_inherited_connections_are_errors() {
    let home = home();
    for (socket, token) in [
        (Some("/tmp/missing"), None),
        (None, Some("a")),
        (Some("relative"), Some("a")),
        (Some("/tmp/missing"), Some("not-a-token")),
    ] {
        let mut cmd = command(home.path());
        if let Some(socket) = socket {
            cmd.env(SOCKET, socket);
        }
        if let Some(token) = token {
            cmd.env(TOKEN, token);
        }
        let output = cmd.arg("sync").output().unwrap();
        assert!(!output.status.success());
    }
}
