fn main() {
    let features: Vec<&str> = vec![
        #[cfg(feature = "plugin")]
        "plugin",
        #[cfg(feature = "sqlite")]
        "sqlite",
        #[cfg(feature = "trash-support")]
        "trash-support",
        #[cfg(feature = "network")]
        "network",
        #[cfg(feature = "mcp")]
        "mcp",
    ];
    println!("cargo:rustc-env=NU_FEATURES={}", features.join(","));

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = std::path::Path::new(&manifest_dir);
    let nu_workspace_path = manifest_path.join("../../../../forks/nushell/Cargo.toml");
    let nu_protocol_path =
        manifest_path.join("../../../../forks/nushell/crates/nu-protocol/Cargo.toml");
    let nu_version = std::fs::read_to_string(&nu_workspace_path)
        .ok()
        .and_then(|contents| find_toml_section_version(&contents, "workspace.package"))
        .or_else(|| {
            std::fs::read_to_string(&nu_protocol_path)
                .ok()
                .and_then(|contents| find_toml_section_version(&contents, "package"))
        })
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=NUSHELL_VERSION={nu_version}");
    println!("cargo:rerun-if-changed=../../../../forks/nushell/Cargo.toml");
    println!("cargo:rerun-if-changed=../../../../forks/nushell/crates/nu-protocol/Cargo.toml");
}

fn find_toml_section_version(contents: &str, section_name: &str) -> Option<String> {
    let wanted_header = format!("[{section_name}]");
    let mut in_section = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == wanted_header;
            continue;
        }
        if in_section {
            if let Some(value) = trimmed
                .strip_prefix("version")
                .and_then(|line| line.trim_start().strip_prefix('='))
                .and_then(|line| line.trim_start().strip_prefix('"'))
            {
                return value.split('"').next().map(String::from);
            }
        }
    }

    None
}
