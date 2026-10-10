#![cfg(unix)]

mod support;

use std::{
    fs::File,
    io::{Read, Write},
    os::{fd::AsRawFd, unix::process::CommandExt},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Terminal {
    child: Child,
    master: File,
    output: String,
}

#[test]
fn parallel_terminal_allocation() {
    std::thread::scope(|scope| {
        for _ in 0..16 {
            scope.spawn(|| {
                for _ in 0..64 {
                    let _pty = support::open_pty(None);
                }
            });
        }
    });
}

#[test]
fn xdg_socket_startup_is_quiet_and_exit_preserves_siblings() {
    for default in [false, true] {
        let home = tempfile::Builder::new()
            .prefix("ntxdg-")
            .tempdir_in("/tmp")
            .unwrap();
        let config = home.path().join("env.nu");
        std::fs::write(&config, "hide-env XDG_DATA_HOME\n").unwrap();
        let mut one = Terminal::start_with_env(home.path(), default.then_some(config.as_path()));
        one.expect("[nu]");
        assert!(!one.output.contains("runtime storage"), "{}", one.output);
        assert!(!one.output.contains("receiver disabled"), "{}", one.output);
        let base = if default {
            home.path().join(".local/share")
        } else {
            home.path().to_path_buf()
        };
        let directory = base.join("astrohacker/nutorch");
        let first = std::fs::read_dir(&directory)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let mut two = Terminal::start_with_env(home.path(), default.then_some(config.as_path()));
        two.expect("[nu]");
        assert!(!two.output.contains("runtime storage"));
        assert!(!two.output.contains("receiver disabled"));
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 2);
        one.send("exit\r");
        let until = Instant::now() + Duration::from_secs(5);
        while one.child.try_wait().unwrap().is_none() {
            assert!(Instant::now() < until);
            one.read_output();
        }
        assert!(!first.exists());
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        two.send("print ('SECOND_' + 'ALIVE')\r");
        two.expect("SECOND_ALIVE");
        two.send("exit\r");
        let until = Instant::now() + Duration::from_secs(5);
        while two.child.try_wait().unwrap().is_none() {
            assert!(Instant::now() < until);
            two.read_output();
        }
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
    }
}

#[test]
fn startup_shows_tip_meta_and_pipes() {
    let home = tempfile::Builder::new()
        .prefix("nttip-")
        .tempdir_in("/tmp")
        .unwrap();
    let mut terminal = Terminal::start(home.path());
    let meta = "Type nutorch tip for more tips.";
    terminal.wait_for("nutorch tip");
    let banner = terminal.output.clone();
    let plain = strip_ansi(&banner);
    let plain_meta = plain.find(meta).unwrap_or_else(|| panic!("{plain:?}"));
    assert!(
        plain[..plain_meta].ends_with("\n\n")
            || plain[..plain_meta].ends_with("\r\n\r\n")
            || plain[..plain_meta].ends_with("\n\r\n"),
        "{plain:?}"
    );
    let raw_at = banner.find("nutorch tip").unwrap();
    let meta_line_start = banner[..raw_at]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    let meta_line_end = banner[raw_at..]
        .find('\n')
        .map(|index| raw_at + index)
        .unwrap_or(banner.len());
    let meta_line = banner[meta_line_start..meta_line_end].trim_end_matches('\r');
    assert_eq!(strip_ansi(meta_line), meta, "{banner:?}");
    assert!(meta_line.contains('\u{1b}'), "{banner:?}");
    let tip_word = banner[..meta_line_start]
        .rfind("Tip")
        .unwrap_or_else(|| panic!("{banner:?}"));
    let label_start = banner[..tip_word]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    let label_end = banner[tip_word..]
        .find('\n')
        .map(|index| tip_word + index)
        .unwrap_or(banner.len());
    let label = &banner[label_start..label_end];
    assert!(label.contains('\u{1b}'), "{banner:?}");
    assert!(strip_ansi(label).starts_with("Tip · "), "{banner:?}");
    let body = &banner[label_end..meta_line_start];
    assert!(body.contains('\u{1b}'), "{banner:?}");
    assert!(
        strip_ansi(body).contains('#'),
        "intro body lost its lesson: {banner:?}"
    );
    terminal.send("nutorch tip --type pipes\r");
    terminal.wait_for("pipelines");
    let shown = terminal.output.clone();
    let at = shown
        .find("pipelines")
        .unwrap_or_else(|| panic!("{shown:?}"));
    let line_start = shown[..at].rfind('\n').map(|index| index + 1).unwrap_or(0);
    let line_end = shown[at..]
        .find('\n')
        .map(|index| at + index)
        .unwrap_or(shown.len());
    let printed_label = &shown[line_start..line_end];
    assert!(printed_label.contains('\u{1b}'), "{shown:?}");
    assert!(
        strip_ansi(printed_label).contains("Tip · nushell · pipelines"),
        "{shown:?}"
    );
    let printed = &shown[line_end..];
    assert!(printed.contains('\u{1b}'), "{shown:?}");
    assert!(
        strip_ansi(printed).contains('#'),
        "pipes body lost its lesson: {shown:?}"
    );
}

// Ignore only color sequences when matching text. Cursor movement and other
// controls separate redraws, so text from two frames cannot form one match.
fn text_match_end(raw: &str, expected: &str) -> Option<usize> {
    let bytes = raw.as_bytes();
    let mut plain = String::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == 0x1b {
            index += 1;
            match *bytes.get(index)? {
                b'[' => {
                    index += 1;
                    while !(0x40..=0x7e).contains(bytes.get(index)?) {
                        index += 1;
                    }
                    let color = bytes[index] == b'm';
                    index += 1;
                    if color {
                        continue;
                    }
                }
                b']' => {
                    index += 1;
                    loop {
                        match *bytes.get(index)? {
                            7 => {
                                index += 1;
                                break;
                            }
                            0x1b if bytes.get(index + 1) == Some(&b'\\') => {
                                index += 2;
                                break;
                            }
                            _ => index += 1,
                        }
                    }
                }
                _ => index += 1,
            }
            plain.push('\0');
        } else {
            let ch = raw[index..].chars().next()?;
            index += ch.len_utf8();
            plain.push(ch);
        }
        if plain.ends_with(expected) {
            return Some(index);
        }
    }
    None
}

#[test]
fn displayed_text_matches_colors_but_not_redraws() {
    let raw = "\x1b]133;A\x1b\\\x1b[32mtorch tenso\x1b[0m\x1b[7;32mr\x1b[0m";
    let end = raw.find("r\x1b[0m").unwrap() + 1;
    for split in 0..end {
        assert_eq!(text_match_end(&raw[..split], "torch tensor"), None);
    }
    assert_eq!(text_match_end(raw, "torch tensor"), Some(end));
    assert_eq!(
        text_match_end("torch tenso\x1b[1;1Hr", "torch tensor"),
        None
    );
    assert_eq!(text_match_end("torch tenso\r\nr", "torch tensor"), None);
    assert_eq!(
        text_match_end("\x1b]0;torch tensor\x07", "torch tensor"),
        None
    );
    let variable = "\x1b[35m$env\x1b[0m.\x1b[32mNAME";
    assert_eq!(text_match_end(variable, "$env.NAME"), Some(variable.len()));
}

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

impl Terminal {
    fn expect_text(&mut self, text: &str) {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if let Some(end) = text_match_end(&self.output, text) {
                self.output.drain(..end);
                return;
            }
            assert!(
                Instant::now() < deadline,
                "waiting for displayed {text:?}: {:?}",
                self.output
            );
            self.read_output();
        }
    }

    fn start(home: &std::path::Path) -> Self {
        Self::start_with_env(home, None)
    }

    fn start_with_env(home: &std::path::Path, env_file: Option<&std::path::Path>) -> Self {
        let size = nix::pty::Winsize {
            ws_row: 40,
            ws_col: 160,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let pty = support::open_pty(Some(&size));
        let slave = File::from(pty.slave);
        let mut command = Command::new(env!("CARGO_BIN_EXE_nutorch"));
        if let Some(env_file) = env_file {
            command.arg("--env-config").arg(env_file);
        } else {
            command.arg("--no-config-file");
        }
        command
            .args(["--no-history", "-i"])
            .env("HOME", home)
            .env("XDG_CONFIG_HOME", home)
            .env("ZDOTDIR", home)
            .env("XDG_DATA_HOME", home)
            .env_remove("NUTORCH_SYNC_SOCKET")
            .env_remove("NUTORCH_SYNC_TOKEN")
            .env_remove("NO_COLOR")
            .env_remove("FORCE_COLOR")
            .env_remove("CLICOLOR")
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    std::path::Path::new(env!("CARGO_BIN_EXE_nutorch"))
                        .parent()
                        .unwrap()
                        .display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("TERM", "xterm-256color")
            .env("PS1", "CHILD> ")
            .stdin(Stdio::from(slave.try_clone().unwrap()))
            .stdout(Stdio::from(slave.try_clone().unwrap()))
            .stderr(Stdio::from(slave));
        // The child owns an isolated controlling terminal, like a terminal tab.
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() == -1 || libc::ioctl(0, libc::TIOCSCTTY as _, 0) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let master = File::from(pty.master);
        unsafe {
            libc::fcntl(master.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK);
        }
        Self {
            child: command.spawn().unwrap(),
            master,
            output: String::new(),
        }
    }

    fn send(&mut self, input: &str) {
        // Let the previous command finish its terminal-mode transition.
        let settle = Instant::now() + Duration::from_millis(150);
        while Instant::now() < settle {
            self.read_output();
        }
        self.output.clear();
        self.master.write_all(input.as_bytes()).unwrap();
    }

    fn wait_for(&mut self, text: &str) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while !self.output.contains(text) {
            assert!(
                Instant::now() < deadline,
                "waiting for {text:?}: {:?}",
                self.output
            );
            self.read_output();
        }
    }

    fn expect(&mut self, text: &str) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while !self.output.contains(text) {
            assert!(
                Instant::now() < deadline,
                "waiting for {text:?}: {:?}",
                self.output
            );
            self.read_output();
        }
        let end = self.output.find(text).unwrap() + text.len();
        self.output.drain(..end);
    }

    fn read_output(&mut self) {
        let mut poll = libc::pollfd {
            fd: self.master.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        if unsafe { libc::poll(&mut poll, 1, 100) } <= 0 {
            return;
        }
        let mut bytes = [0; 8192];
        let count = match self.master.read(&mut bytes) {
            Ok(count) => count,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return,
            Err(error) if error.raw_os_error() == Some(libc::EIO) => return,
            Err(error) => panic!("terminal read failed: {error}; {:?}", self.output),
        };
        if count == 0 {
            return;
        }
        self.output
            .push_str(&String::from_utf8_lossy(&bytes[..count]));
        // Emulate the cursor-position reply needed by Reedline's painter.
        if self.output.contains("\x1b[6n") {
            self.master.write_all(b"\x1b[1;1R").unwrap();
            self.output = self.output.replace("\x1b[6n", "");
        }
    }
}

#[test]
fn sync_codec_aliases_loaded_at_startup_import_child_environment() {
    for shell in ["/bin/zsh -f", "/bin/bash --noprofile --norc"] {
        let home = tempfile::Builder::new()
            .prefix("ntalias-")
            .tempdir_in("/tmp")
            .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let env_file = home.path().join("env.nu");
        std::fs::write(
            &env_file,
            r#"
            if ($env.PATH | describe) == 'string' { $env.PATH = ($env.PATH | split row ':') }
            $env.ENV_CONVERSIONS = {
                PATH: {from_string: {|v| $v | split row ':'}, to_string: {|v| $v | str join ':'}},
                Path: {from_string: {|v| $v | split row ':'}, to_string: {|v| $v | str join ':'}}
            }
            print ('ALIAS_ENV_' + 'LOADED')
        "#,
        )
        .unwrap();
        let mut terminal = Terminal::start_with_env(home.path(), Some(&env_file));
        terminal.expect("ALIAS_ENV_LOADED");
        terminal.expect("[nu]");
        terminal.send(&format!("let original_path = $env.PATH; {shell}\r"));
        terminal.expect("CHILD> ");
        // PATH must change; otherwise equal-value preservation skips both codecs.
        terminal
            .send("export MYTESTVAR=test; export PATH=\"$PATH:/alias-regression\"; nutorch sync\r");
        terminal.expect("Queued environment snapshot");
        terminal.send("exit\r");
        let deadline = Instant::now() + Duration::from_secs(20);
        while !terminal.output.contains("[nu]") {
            assert!(Instant::now() < deadline, "parent prompt did not return");
            terminal.read_output();
        }
        assert!(
            !terminal
                .output
                .contains("ambiguous receiver environment codecs"),
            "snapshot rejected: ambiguous receiver environment codecs; ordinary variable cannot be imported"
        );
        terminal.expect("[nu]");
        terminal.send("if $env.MYTESTVAR == 'test' and $env.PATH == ($original_path | append '/alias-regression') { print ('ALIAS_' + 'IMPORTED') }; ^printf 'EXTERNAL_%s' OK\r");
        terminal.expect("ALIAS_IMPORTED");
        terminal.expect("EXTERNAL_OK");
    }
}

#[test]
fn sync_ordinary_shells_and_fullscreen_terminal_return() {
    let editor = Command::new("vim")
        .arg("--version")
        .output()
        .expect("Fullscreen tests require vim on PATH");
    assert!(editor.status.success(), "vim --version failed");
    for shell in ["/bin/zsh -f", "/bin/bash --noprofile --norc"] {
        let home = tempfile::Builder::new()
            .prefix("ntpty-")
            .tempdir_in("/tmp")
            .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let document = home.path().join("pages");
        std::fs::write(
            &document,
            (0..200).map(|i| format!("line {i}\n")).collect::<String>(),
        )
        .unwrap();
        let mut terminal = Terminal::start(home.path());
        terminal.expect("[nu]");
        terminal.send("$env.SYNC_VALUE = 'old'; $env.TYPED_SYNC = 42; $env.config.hooks.pre_prompt = [{ if $env.SYNC_VALUE == 'child' { print ('HOOK_' + 'UPDATED') } }]; print ('SETUP_' + 'DONE')\r");
        terminal.expect("SETUP_DONE");
        terminal.send(&format!(
            "{shell}; print ('SAME_LINE_' + $env.SYNC_VALUE)\r"
        ));
        terminal.expect("CHILD> ");
        terminal.send("export SYNC_VALUE=child; LOCAL_ONLY=private; nutorch sync; printf 'CHILD_%s\\n' ready\r");
        terminal.expect("Queued environment snapshot");
        terminal.expect("CHILD_ready");
        for command in [
            "vim -Nu NONE -n -i NONE".to_string(),
            format!("less '{}'", document.display()),
        ] {
            terminal.send(&format!("{command}\r"));
            terminal.expect("\x1b[?1049h");
            // The foreground group must belong to the child TUI, not NuTorch.
            let group = unsafe { libc::tcgetpgrp(terminal.master.as_raw_fd()) };
            assert!(group > 0);
            assert_ne!(group, terminal.child.id() as i32);
            terminal.send(if command.starts_with("vim") {
                ":q!\r"
            } else {
                "q"
            });
            terminal.expect("\x1b[?1049l");
            terminal.send("printf 'TUI_%s\\n' returned\r");
            terminal.expect("TUI_returned");
        }
        terminal.send("sleep 30\r");
        std::thread::sleep(Duration::from_millis(250));
        terminal.send("\x1a");
        terminal.send("fg\r");
        std::thread::sleep(Duration::from_millis(250));
        terminal.send("\x03");
        terminal.send("printf 'SIGNAL_%s\\n' recovered\r");
        terminal.expect("SIGNAL_recovered");
        terminal.send("exit\r");
        terminal.expect("SAME_LINE_old");
        terminal.expect("HOOK_UPDATED");
        terminal.expect("[nu]");
        terminal.send("print ('VALUE_' + $env.SYNC_VALUE); print ($env.TYPED_SYNC | describe); if 'LOCAL_ONLY' not-in $env { print ('LOCAL_' + 'ABSENT') }; print ('SYNC_' + 'RETURNED')\r");
        terminal.expect("VALUE_child");
        terminal.expect("int");
        terminal.expect("LOCAL_ABSENT");
        terminal.expect("SYNC_RETURNED");
        assert_eq!(
            unsafe { libc::tcgetpgrp(terminal.master.as_raw_fd()) },
            terminal.child.id() as i32
        );
        terminal.send("exit\r");
        let until = Instant::now() + Duration::from_secs(5);
        while terminal.child.try_wait().unwrap().is_none() {
            assert!(Instant::now() < until);
            terminal.read_output();
        }
        let sockets = home.path().join("astrohacker/nutorch");
        assert_eq!(std::fs::read_dir(sockets).unwrap().count(), 0);
    }
}

#[test]
fn sync_nested_sessions_route_only_when_explicit() {
    let home = tempfile::Builder::new()
        .prefix("ntnest-")
        .tempdir_in("/tmp")
        .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut terminal = Terminal::start(home.path());
    terminal.expect("[nu]");
    terminal.send("$env.NESTED_SYNC = 'outer'; nutorch --no-config-file --no-history\r");
    terminal.expect("Startup Time:");
    terminal.expect("[nu]");
    terminal.send("/bin/zsh -f\r");
    terminal.expect("CHILD> ");
    terminal.send("export NESTED_SYNC=inner; nutorch sync\r");
    terminal.expect("Queued environment snapshot");
    terminal.send("exit\r");
    terminal.expect("[nu]");
    terminal.send("print ('B_' + $env.NESTED_SYNC)\r");
    terminal.expect("B_inner");
    terminal.send("exit\r");
    terminal.expect("[nu]");
    terminal.send("print ('A_' + $env.NESTED_SYNC)\r");
    terminal.expect("A_outer");
    terminal.send("nutorch --no-config-file --no-history\r");
    terminal.expect("Startup Time:");
    terminal.expect("[nu]");
    terminal.send("$env.NESTED_SYNC = 'forwarded'; use torch nutorch; nutorch sync\r");
    terminal.expect("Queued environment snapshot");
    terminal.send("exit\r");
    terminal.expect("[nu]");
    terminal.send("print ('A_' + $env.NESTED_SYNC); nutorch sync\r");
    terminal.expect("A_forwarded");
    terminal.expect("no enclosing NuTorch sync receiver");
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.child.kill();
        // macOS can defer reaping until the PTY output has drained.
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            self.read_output();
            if self.child.try_wait().ok().flatten().is_some() {
                break;
            }
        }
    }
}

#[test]
fn sync_prompt_boundary_hooks_conversions_and_exit_status() {
    let home = tempfile::Builder::new()
        .prefix("ntboundary-")
        .tempdir_in("/tmp")
        .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut terminal = Terminal::start(home.path());
    terminal.expect("[nu]");
    terminal.send("$env.QUEUED = 'old'; $env.config.hooks.env_change.QUEUED = [{|before, after| print ('CHANGE_' + $after) }]; print ('BOUNDARY_' + 'READY')\r");
    terminal.expect("BOUNDARY_READY");
    terminal.send("with-env {QUEUED: 'new'} { ^nutorch sync }; print ('LINE_' + $env.QUEUED); /bin/sh -c 'exit 7'\r");
    terminal.expect("Queued environment snapshot");
    terminal.expect("LINE_old");
    terminal.expect("CHANGE_new");
    terminal.expect("[nu]");
    terminal.send("print ('STATUS_' + ($env.LAST_EXIT_CODE | into string)); print ('PROMPT_' + $env.QUEUED)\r");
    terminal.expect("STATUS_7");
    terminal.expect("PROMPT_new");
    // Unchanged lists retain their type; changed strings use the receiver codec.
    terminal.send("$env.CONVERTED = [a b]; $env.ENV_CONVERSIONS.CONVERTED = {to_string: {|v| $v | str join ':'}, from_string: {|v| $v | split row ':'}}; ^nutorch sync\r");
    terminal.expect("Queued environment snapshot");
    terminal
        .send("print ($env.CONVERTED | describe); with-env {CONVERTED: [c d]} { ^nutorch sync }\r");
    terminal.expect("list<string>");
    terminal.expect("Queued environment snapshot");
    terminal.send("print ($env.CONVERTED | describe); print ('CONVERTED_' + ($env.CONVERTED | str join ':'))\r");
    terminal.expect("list<string>");
    terminal.expect("CONVERTED_c:d");
    // Receiver conversion failure rejects all included updates, including SAFE.
    terminal.send("hide-env CONVERTED; $env.ENV_CONVERSIONS = {}; $env.SAFE = 'old'; $env.BROKEN = 'old'; $env.ENV_CONVERSIONS.BROKEN = {from_string: {|v| $v}, to_string: {|v| error make {msg: 'SECRET_CONVERSION_DETAIL'}}}; print ('FAILURE_' + 'READY')\r");
    terminal.expect("FAILURE_READY");
    terminal
        .send("with-env {ENV_CONVERSIONS: {}, SAFE: 'changed', BROKEN: 'new'} { ^nutorch sync }\r");
    terminal.expect("Queued environment snapshot");
    terminal.expect("snapshot");
    terminal.expect("snapshot rejected");
    assert!(!terminal.output.contains("SECRET_CONVERSION_DETAIL"));
    terminal.send("print ('SAFE_' + $env.SAFE)\r");
    terminal.expect("SAFE_old");
}

#[test]
fn sync_failed_initialization_does_not_route_to_ancestor() {
    let home = tempfile::Builder::new()
        .prefix("ntfail-")
        .tempdir_in("/tmp")
        .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let bad = home.path().join("unsafe");
    let private = bad.join("astrohacker/nutorch");
    std::fs::create_dir_all(&private).unwrap();
    std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut terminal = Terminal::start(home.path());
    terminal.expect("[nu]");
    terminal.send(&format!(
        "with-env {{XDG_DATA_HOME: '{}'}} {{ nutorch --no-config-file --no-history }}\r",
        bad.display()
    ));
    terminal.expect("receiver disabled");
    terminal.expect("[nu]");
    terminal.send("if 'NUTORCH_SYNC_SOCKET' not-in $env and 'NUTORCH_SYNC_TOKEN' not-in $env { print ('CONNECTION_' + 'CLEARED') }; nutorch sync\r");
    terminal.expect("CONNECTION_CLEARED");
    terminal.expect("sync is disabled");
    terminal.send("^nutorch sync\r");
    terminal.expect("no enclosing NuTorch sync receiver");
    terminal.send("exit\r");
    terminal.expect("[nu]");
    terminal.send("print ('ANCESTOR_' + 'USABLE')\r");
    terminal.expect("ANCESTOR_USABLE");
}

#[test]
fn sync_receiver_codecs_work_in_real_child_and_nested_shells() {
    for shell in ["/bin/zsh -f", "/bin/bash --noprofile --norc"] {
        let home = tempfile::Builder::new()
            .prefix("ntcodec-")
            .tempdir_in("/tmp")
            .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut terminal = Terminal::start(home.path());
        terminal.expect("[nu]");
        terminal.send("$env.CODEC_ITEMS = [old]; $env.CODEC_RECORD = {old: 1}; $env.ENV_CONVERSIONS = {CODEC_ITEMS: {to_string: {|v| $v | str join '|'}, from_string: {|v| $v | split row '|'}}, CODEC_RECORD: {to_string: {|v| $v | to json --raw}, from_string: {|v| $v | from json}}, PATH: {to_string: {|v| $v | str join ':'}, from_string: {|v| $v | split row ':' | prepend '/codec'}}}; $env.config.hooks.pre_prompt = [{ if $env.CODEC_ITEMS == [child typed] { print ('CODEC_HOOK_' + 'TYPED') } }]; print ('CODEC_' + 'READY')\r");
        terminal.expect("CODEC_READY");
        terminal.send(&format!(
            "{shell}; print ('CODEC_LINE_' + ($env.CODEC_ITEMS | str join '|'))\r"
        ));
        terminal.expect("CHILD> ");
        terminal.send("export CODEC_ITEMS='child|typed'; export CODEC_RECORD='{\"answer\":42}'; export PATH=\"$PATH:/incoming\"; nutorch sync\r");
        terminal.expect("Queued environment snapshot");
        terminal.send("exit\r");
        terminal.expect("CODEC_LINE_old");
        terminal.expect("CODEC_HOOK_TYPED");
        terminal.send("if ($env.CODEC_ITEMS | describe) == 'list<string>' and $env.CODEC_RECORD.answer == 42 and $env.PATH.0 == '/codec' and ($env.PATH | last) == '/incoming' { print ('CODEC_' + 'EXACT') }\r");
        terminal.expect("CODEC_EXACT");
        terminal.send("nutorch --no-config-file --no-history\r");
        terminal.expect("Startup Time:");
        terminal.expect("[nu]");
        terminal.send("$env.CODEC_ITEMS = 'nested|typed'; nutorch sync\r");
        terminal.expect("Queued environment snapshot");
        terminal.send("exit\r");
        terminal.expect("[nu]");
        terminal
            .send("if $env.CODEC_ITEMS == [nested typed] { print ('NESTED_CODEC_' + 'TYPED') }\r");
        terminal.expect("NESTED_CODEC_TYPED");
    }
}

#[test]
fn sync_codec_cancellation_rejects_batch_and_prompt_recovers() {
    let home = tempfile::Builder::new()
        .prefix("ntcancel-")
        .tempdir_in("/tmp")
        .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut terminal = Terminal::start(home.path());
    terminal.expect("[nu]");
    terminal.send("$env.SAFE = 'old'; $env.ENV_CONVERSIONS = {SLOW: {from_string: {|v| print ('DECODER_' + 'RUNNING'); loop {} }}}; print ('CANCEL_' + 'READY')\r");
    terminal.expect("CANCEL_READY");
    terminal.send("with-env {SLOW: 'start', SAFE: 'new'} { ^nutorch sync }\r");
    terminal.expect("Queued environment snapshot");
    terminal.expect("DECODER_RUNNING");
    terminal.send("\x03");
    terminal.expect("snapshot rejected");
    terminal.expect("[nu]");
    terminal.send("if $env.SAFE == 'old' and 'SLOW' not-in $env { print ('CANCEL_' + 'ATOMIC') }; with-env {SAFE: 'recovered'} { ^nutorch sync }\r");
    terminal.expect("CANCEL_ATOMIC");
    terminal.expect("Queued environment snapshot");
    terminal.send("print ('CANCEL_' + $env.SAFE)\r");
    terminal.expect("CANCEL_recovered");
}

#[test]
fn sync_while_editing_waits_and_killed_sessions_leave_isolated_stale_sockets() {
    let home = tempfile::Builder::new()
        .prefix("ntedit-")
        .tempdir_in("/tmp")
        .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut terminal = Terminal::start(home.path());
    terminal.expect("[nu]");
    // Private, ephemeral test rendezvous, removed immediately after reading.
    let rendezvous = home.path().join("connection.json");
    terminal.send(&format!("$env.EDIT_SYNC = 'old'; [$env.NUTORCH_SYNC_SOCKET $env.NUTORCH_SYNC_TOKEN] | to json | save '{}'; print ('EDIT_' + 'READY')\r",rendezvous.display()));
    terminal.expect("EDIT_READY");
    let connection: Vec<String> =
        serde_json::from_slice(&std::fs::read(&rendezvous).unwrap()).unwrap();
    std::fs::remove_file(rendezvous).unwrap();
    let endpoint = nutorch::sync::transport::Endpoint {
        path: connection[0].clone().into(),
        token: connection[1].clone(),
    };
    terminal.send("print ('EDIT_' + $env.EDIT_SYNC)");
    nutorch::sync::transport::send(
        &endpoint,
        vec![
            ("EDIT_SYNC".into(), "new".into()),
            ("NUTORCH_TEST_SYNC_COMPLETION".into(), "completed".into()),
        ],
    )
    .unwrap();
    terminal.send("\r");
    terminal.expect("EDIT_old");
    terminal.send("print ('NEXT_' + $env.EDIT_SYNC)\r");
    terminal.expect("NEXT_new");
    terminal.send("$env.NUTORCH_TEST_SYNC_COMPL\t");
    terminal.expect_text("$env.NUTORCH_TEST_SYNC_COMPLETION");
    terminal.send("\r");
    terminal.expect("completed");
    terminal.child.kill().unwrap();
    let until = Instant::now() + Duration::from_secs(3);
    while terminal.child.try_wait().unwrap().is_none() {
        assert!(Instant::now() < until);
        terminal.read_output();
    }
    assert!(endpoint.path.exists());
    assert!(nutorch::sync::transport::send(&endpoint, vec![]).is_err());
    let mut sibling = Terminal::start(home.path());
    sibling.expect("[nu]");
    assert_eq!(
        std::fs::read_dir(home.path().join("astrohacker/nutorch"))
            .unwrap()
            .count(),
        2
    );
    sibling.send("exit\r");
    let until = Instant::now() + Duration::from_secs(3);
    while sibling.child.try_wait().unwrap().is_none() {
        assert!(Instant::now() < until);
        sibling.read_output();
    }
    assert!(endpoint.path.exists());
    assert_eq!(
        std::fs::read_dir(home.path().join("astrohacker/nutorch"))
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn native_tensor_work_cancels_and_shell_remains_usable() {
    let home = tempfile::tempdir().unwrap();
    let mut terminal = Terminal::start(home.path());
    terminal.expect("[nu]");
    terminal.send(
        "use torch; let x = (torch tensor [1 2 3] --requires_grad); print ('TENSOR_' + 'READY')\r",
    );
    terminal.expect("TENSOR_READY");
    terminal.send("$x | torch mul $x | torch sum | torch backward; print ('LOOP_' + 'STARTED'); loop { $x | torch mul $x | torch sum | torch backward; torch zero_grad $x }\r");
    terminal.expect("LOOP_STARTED");
    // Give the loop time to submit several GPU operations before cancelling.
    let until = Instant::now() + Duration::from_millis(500);
    while Instant::now() < until {
        terminal.read_output();
    }
    terminal.send("\x03");
    terminal.expect("[nu]");
    terminal.send(
        "print ($x | torch mul $x | torch tolist | to json --raw); print ('TENSOR_' + 'RECOVERED')\r",
    );
    terminal.expect("[1.0,4.0,9.0]");
    terminal.expect("TENSOR_RECOVERED");
}

#[test]
fn native_optimizer_training_cancels_and_existing_aliases_recover() {
    let home = tempfile::tempdir().unwrap();
    let mut terminal = Terminal::start(home.path());
    terminal.expect("[nu]");
    terminal.send("use torch; let model = torch nn linear 1 1 --no-bias --weight (torch tensor [[0.0]]); let held = (torch nn parameters $model).0; let opt = torch nn sgd $model --lr 0.01 --momentum 0.5; let x = torch tensor [[1.0]]; let target = torch tensor [[2.0]]; print ('MODEL_' + 'READY')\r");
    terminal.expect("MODEL_READY");
    terminal.send("$x | torch forward $model | torch mse_loss $target | torch backward; torch step $opt; torch nn zero_grad $opt; print ('TRAIN_' + 'STARTED'); loop { $x | torch forward $model | torch mse_loss $target | torch backward; torch step $opt; torch nn zero_grad $opt }\r");
    terminal.expect("TRAIN_STARTED");
    let until = Instant::now() + Duration::from_millis(500);
    while Instant::now() < until {
        terminal.read_output();
    }
    terminal.send("\x03");
    terminal.expect("[nu]");
    // Cancellation can occur between backward, step and zero_grad. Clear any
    // retained gradient explicitly, then reuse the same model and optimizer.
    terminal.send("torch nn zero_grad $opt; $x | torch forward $model | torch mse_loss $target | torch backward; torch step $opt; torch nn zero_grad $opt; let prediction = ($x | torch forward $model | torch value).0.0; let weight = ($held | torch value).0.0; if ($prediction == $weight) and ($weight > 0.0) and ($weight <= 2.01) { print ('TRAIN_' + 'RECOVERED') }\r");
    terminal.expect("TRAIN_RECOVERED");
}

#[test]
fn torch_import_enables_interactive_completion_and_help() {
    check_torch_completion(true);
    check_torch_completion(false);
}

fn check_torch_completion(quick: bool) {
    let home = tempfile::tempdir().unwrap();
    let mut terminal = Terminal::start(home.path());
    terminal.expect("[nu]");
    terminal.send("if (scope commands | where name == 'torch tensor' | is-empty) { print ('IMPORT_' + 'ABSENT') }\r");
    terminal.expect("IMPORT_ABSENT");
    terminal.send("use torch; print ('IMPORT_' + 'READY')\r");
    terminal.expect("IMPORT_READY");
    terminal.send(&format!(
        "$env.config.completions.quick = {quick}; $env.config.completions.partial = {quick}; print ('COMPLETION_' + 'CONFIGURED')\r"
    ));
    terminal.expect("COMPLETION_CONFIGURED");
    terminal.send("help torch tensor\r");
    terminal.expect("Create a native MPS tensor");
    terminal.expect("[nu]");
    terminal.send("torch tenso\t");
    terminal.expect_text("torch tensor");
    // Upstream now accepts a lone asynchronous result when quick completion is
    // enabled. With quick and partial insertion disabled, Enter accepts the
    // open menu selection rather than submitting an already-completed line.
    if !quick {
        terminal.send("\r");
        terminal.expect_text("torch tensor");
    }
    terminal.send(" [2 3] | torch value | to json --raw\r");
    terminal.expect("[2.0,3.0]");
}

#[test]
fn shift_tab_ai_input_never_runs_and_return_clears_buffer() {
    let home = tempfile::tempdir().unwrap();
    let startup = home.path().join("zsh-started");
    for name in [".zshenv", ".zprofile", ".zshrc", ".zlogin"] {
        std::fs::write(
            home.path().join(name),
            format!("touch '{}'\n", startup.display()),
        )
        .unwrap();
    }
    let sentinel = home.path().join("executed");
    let command = format!("touch '{}'", sentinel.display());
    let mut terminal = Terminal::start(home.path());
    terminal.expect("[nu]");
    terminal.send("\x1b[Z");
    terminal.expect("[ai]");
    // Check live rendering before Enter; submitted-line output alone can hide
    // a highlighter that suppresses the editable buffer.
    terminal.send("VISIBLE_AI_PROSE");
    terminal.expect("VISIBLE_AI_PROSE");
    terminal.send("\r");
    terminal.expect("input was not executed");
    terminal.expect("[ai]");
    terminal.send(&format!("{command}\r"));
    terminal.expect("input was not executed");
    terminal.expect("[ai]");
    assert!(!sentinel.exists());
    // Incomplete Nushell syntax is still submitted as plain AI text.
    terminal.send("please explain {\r");
    terminal.expect("input was not executed");
    terminal.expect("[ai]");
    terminal.send(&command);
    terminal.expect("touch");
    terminal.send("\x1b[Z");
    terminal.expect("[nu]");
    terminal.send("print ('CLEARED_' + 'OK')\r");
    terminal.expect("CLEARED_OK");
    assert!(!sentinel.exists(), "AI buffer leaked into Nushell");
    terminal.send(&format!("{command}; print ('EXECUTED_' + 'OK')\r"));
    terminal.expect("EXECUTED_OK");
    assert!(sentinel.exists(), "normal Nushell did not execute");
    assert!(!startup.exists(), "a zsh startup file was read");
    terminal.send("exit\r");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = terminal.child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        assert!(Instant::now() < deadline, "shell did not exit");
        terminal.read_output();
    }
}
