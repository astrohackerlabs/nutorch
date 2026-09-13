#![cfg(unix)]

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

impl Terminal {
    fn start(home: &std::path::Path) -> Self {
        let size = nix::pty::Winsize {
            ws_row: 40,
            ws_col: 160,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let pty = nix::pty::openpty(Some(&size), None).unwrap();
        let slave = File::from(pty.slave);
        let mut command = Command::new(env!("CARGO_BIN_EXE_nutorch"));
        command
            .args(["--no-config-file", "--no-history", "-i"])
            .env("HOME", home)
            .env("XDG_CONFIG_HOME", home)
            .env("ZDOTDIR", home)
            .env("TERM", "xterm-256color")
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
        self.output.clear();
        self.master.write_all(input.as_bytes()).unwrap();
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
    let home = tempfile::tempdir().unwrap();
    let mut terminal = Terminal::start(home.path());
    terminal.expect("[nu]");
    terminal.send("if (scope commands | where name == 'torch tensor' | is-empty) { print ('IMPORT_' + 'ABSENT') }\r");
    terminal.expect("IMPORT_ABSENT");
    terminal.send("use torch; print ('IMPORT_' + 'READY')\r");
    terminal.expect("IMPORT_READY");
    terminal.send("help torch tensor\r");
    terminal.expect("Create a native MPS tensor");
    terminal.expect("[nu]");
    terminal.send("torch tenso\t");
    terminal.expect("torch tensor");
    // Enter accepts the selected completion menu entry; the next Enter runs it.
    terminal.send("\r");
    terminal.expect("torch tensor");
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
