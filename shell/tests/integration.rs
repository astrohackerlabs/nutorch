//! Integration tests for the shipped `nutorch` binary (real entry path).

use std::process::Command;

fn nutorch() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_nutorch"));
    command.arg("--no-config-file");
    command
}

#[test]
fn package_version_ignores_termsurf_version() {
    let output = nutorch()
        .env("ASTROHACKER_VERSION", "99.99.99")
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("nutorch {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn structured_pipelines_and_comments_remain_nushell() {
    let output = nutorch()
        .args([
            "-c",
            "# a Nushell comment\n[1 2 3] | each {|n| $n * 2} | to json --raw",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "[2,4,6]");
}

#[test]
fn script_arguments_external_commands_and_inherited_environment() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("arguments with spaces.nu");
    std::fs::write(
        &script,
        "def main [value: string] { ^printf '%s:%s' $value $env.NUTORCH_TEST_PARENT }",
    )
    .unwrap();
    let output = nutorch()
        .env("NUTORCH_TEST_PARENT", "inherited")
        .arg(script)
        .arg("two words")
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(output.stdout, b"two words:inherited");
}

#[test]
fn explicit_config_and_env_files_load() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.nu");
    let env = dir.path().join("env.nu");
    std::fs::write(&config, "$env.NUTORCH_TEST_CONFIG = 'config'").unwrap();
    std::fs::write(&env, "$env.NUTORCH_TEST_ENV = 'env'").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_nutorch"))
        .arg("--config")
        .arg(config)
        .arg("--env-config")
        .arg(env)
        .args([
            "-c",
            "print ($env.NUTORCH_TEST_CONFIG + ':' + $env.NUTORCH_TEST_ENV)",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(output.stdout, b"config:env\n");
}

#[test]
fn exit_command_propagates_code_7() {
    let output = nutorch()
        .args(["-c", "exit 7"])
        .output()
        .expect("spawn nutorch");
    assert_eq!(
        output.status.code(),
        Some(7),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Exit doesn't catch"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn exit_command_propagates_code_0() {
    let status = nutorch()
        .args(["-c", "exit 0"])
        .status()
        .expect("spawn nutorch");
    assert_eq!(status.code(), Some(0));
}

#[test]
fn exit_command_propagates_code_42() {
    let status = nutorch()
        .args(["-c", "exit 42"])
        .status()
        .expect("spawn nutorch");
    assert_eq!(status.code(), Some(42));
}
