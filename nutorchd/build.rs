//! Stamp the short git sha into the binary (issue 0011). Falls back to
//! "unknown" outside a git checkout (source tarballs, brew builds).

fn main() {
    // Re-run when the commit changes, not just on source edits.
    println!("cargo:rerun-if-changed=../.git/HEAD");
    if let Ok(head) = std::fs::read_to_string("../.git/HEAD") {
        if let Some(reference) = head.strip_prefix("ref: ") {
            println!("cargo:rerun-if-changed=../.git/{}", reference.trim());
        }
    }
    let sha = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=NUTORCH_GIT_SHA={sha}");
}
