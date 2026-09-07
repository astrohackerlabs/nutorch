use std::process::Command;

fn assert_version(binary: &std::path::Path, argument: &str) {
    let output = Command::new(binary).arg(argument).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        concat!("nutorch ", env!("CARGO_PKG_VERSION"), "\n").as_bytes()
    );
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
}

#[test]
fn version_contains_only_package_version() {
    let binary = std::path::Path::new(env!("CARGO_BIN_EXE_torch"));
    assert_version(binary, "--version");
    assert_version(binary, "version");
}

#[cfg(unix)]
#[test]
fn installed_name_alias_has_identical_version() {
    let directory = std::env::temp_dir().join(format!(
        "nutorch-version-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let alias = directory.join("nutorch");
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_torch"), &alias).unwrap();
    assert_version(&alias, "--version");
    std::fs::remove_file(alias).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
