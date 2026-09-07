//! Stamp the short git sha into the binary (issue 0011). Falls back to
//! "unknown" outside a git checkout (source tarballs, brew builds).

fn main() {
    // Resolve Git's own paths: .git may be a worktree file, and refs may
    // live in the common directory or packed-refs rather than beside HEAD.
    let mut paths = vec!["HEAD".to_string(), "packed-refs".to_string()];
    if let Some(reference) = git(&["symbolic-ref", "-q", "HEAD"]) {
        paths.push(reference);
    }
    for path in paths {
        if let Some(resolved) = git(&["rev-parse", "--git-path", &path]) {
            println!("cargo:rerun-if-changed={resolved}");
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

fn git(args: &[&str]) -> Option<String> {
    std::process::Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}
