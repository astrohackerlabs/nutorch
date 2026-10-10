#[cfg(unix)]
pub fn open_pty(size: Option<&nix::pty::Winsize>) -> nix::pty::OpenptyResult {
    // Concurrent openpty calls can fail with OS error -6 on macOS 27.
    // Serialize allocation only. Keep shell execution parallel and report errors.
    static ALLOCATION: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = ALLOCATION.lock().unwrap();
    nix::pty::openpty(size, None).unwrap_or_else(|error| {
        panic!(
            "openpty: {error}; OS error: {:?}",
            std::io::Error::last_os_error()
        )
    })
}
