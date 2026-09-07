use std::process::Command;

#[test]
fn version_is_package_only_and_exits_before_daemon_setup() {
    let output = Command::new(env!("CARGO_BIN_EXE_nutorchd"))
        .args([
            "--version",
            "--socket",
            "/nonexistent/nutorch-version-test/socket",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        concat!("nutorch ", env!("CARGO_PKG_VERSION"), "\n").as_bytes()
    );
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
}
