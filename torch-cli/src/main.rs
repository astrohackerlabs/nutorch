//! torch: the Nutorch v2 thin client (PoC, issue 0002).
//!
//! One operation per invocation: build a one-line JSON request, send it to
//! nutorchd over the Unix socket, print the result. Handles print as bare
//! strings so they compose in POSIX pipelines; when an op needs a tensor
//! handle and none is given positionally, one line is read from stdin (the
//! dual input pattern's pipeline form). Deliberately has no tch dependency —
//! the client stays thin.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};
use std::time::Duration;

struct Args {
    op: String,
    positional: Vec<String>,
    dtype: Option<String>,
    socket: Option<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut raw = std::env::args().skip(1);
    let op = raw.next().ok_or("usage: torch <op> [args...]")?;
    let mut positional = Vec::new();
    let mut dtype = None;
    let mut socket = None;
    while let Some(arg) = raw.next() {
        match arg.as_str() {
            "--dtype" => dtype = Some(raw.next().ok_or("--dtype needs a value")?),
            "--socket" => socket = Some(raw.next().ok_or("--socket needs a value")?),
            "--device" => {
                return Err(
                    "the device option was removed; tensors always live on the GPU (mps)"
                        .to_string(),
                )
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag: {flag}")),
            _ => positional.push(arg),
        }
    }
    Ok(Args {
        op,
        positional,
        dtype,
        socket,
    })
}

fn default_socket_path() -> PathBuf {
    match std::env::var_os("TMPDIR") {
        Some(tmp) => PathBuf::from(tmp).join("nutorchd.sock"),
        None => PathBuf::from("/tmp/nutorchd.sock"),
    }
}

/// Liveness probe that DROPS the probe connection before returning.
///
/// The drop matters: holding a probe stream (e.g. as a `match` scrutinee,
/// whose temporary lives for the whole match arm) while sending a follow-up
/// request on a second connection deadlocks against the serial
/// one-connection-at-a-time daemon — it is still waiting on the open probe.
fn daemon_alive(socket: &Path) -> bool {
    UnixStream::connect(socket).is_ok()
}

/// The socket path with its extension replaced by `.log`
/// (nutorchd.sock -> nutorchd.log) — must agree with the daemon's convention.
fn log_path_for(socket: &Path) -> PathBuf {
    socket.with_extension("log")
}

/// Locate the daemon binary: `NUTORCHD_BIN` override, else next to `torch`.
fn nutorchd_binary() -> PathBuf {
    if let Some(bin) = std::env::var_os("NUTORCHD_BIN") {
        return PathBuf::from(bin);
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("nutorchd")))
        .unwrap_or_else(|| PathBuf::from("nutorchd"))
}

/// Auto-start (issue 0004): spawn nutorchd detached — stdin null,
/// stdout/stderr appended to the conventional log file — then poll the
/// socket until it answers. The daemon's probe-bind makes spawn races
/// harmless (losers exit 0), modulo the simultaneous-start window recorded
/// in issue 0004 experiment 1.
fn ensure_daemon(socket: &Path) -> Result<(), String> {
    if daemon_alive(socket) {
        return Ok(());
    }
    let binary = nutorchd_binary();
    let log = log_path_for(socket);
    let open_log = || {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log)
            .map_err(|e| format!("cannot open daemon log {}: {e}", log.display()))
    };
    std::process::Command::new(&binary)
        .arg("--socket")
        .arg(socket)
        .stdin(Stdio::null())
        .stdout(open_log()?)
        .stderr(open_log()?)
        .spawn()
        .map_err(|e| format!("failed to start nutorchd ({}): {e}", binary.display()))?;
    for _ in 0..100 {
        if daemon_alive(socket) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(format!(
        "nutorchd did not come up within 5s; see {}",
        log.display()
    ))
}

/// Positional argument if present, else one line from stdin (pipeline form).
fn positional_or_stdin(args: &Args, index: usize, what: &str) -> Result<String, String> {
    if let Some(value) = args.positional.get(index) {
        return Ok(value.clone());
    }
    let mut line = String::new();
    std::io::stdin()
        .read_to_string(&mut line)
        .map_err(|e| format!("failed to read {what} from stdin: {e}"))?;
    let line = line.trim();
    if line.is_empty() {
        return Err(format!(
            "missing {what}: pass it as an argument or pipe it in"
        ));
    }
    Ok(line.to_string())
}

/// Two handles for a binary op. Two positionals → (a, b). One positional →
/// a comes from stdin and the positional is b (pipeline form, matching v1's
/// pipeline-is-first-operand convention). Zero positionals → error.
fn binary_handles(args: &Args, op: &str) -> Result<(String, String), String> {
    match args.positional.len() {
        2 => Ok((args.positional[0].clone(), args.positional[1].clone())),
        1 => {
            let a = positional_or_stdin(args, 1, "left-hand tensor handle")?;
            Ok((a, args.positional[0].clone()))
        }
        0 => Err(format!(
            "{op} needs two tensor handles (two arguments, or pipe one in and pass one)"
        )),
        n => Err(format!("{op} takes two tensor handles, got {n} arguments")),
    }
}

fn build_request(args: &Args) -> Result<serde_json::Value, String> {
    match args.op.as_str() {
        "tensor" => {
            let data_text = positional_or_stdin(args, 0, "tensor data")?;
            let data: serde_json::Value = serde_json::from_str(&data_text)
                .map_err(|e| format!("tensor data is not valid JSON: {e}"))?;
            Ok(serde_json::json!({
                "op": "tensor",
                "data": data,
                "dtype": args.dtype,
            }))
        }
        "full" => {
            let shape_text = args
                .positional
                .first()
                .ok_or("full needs a shape, e.g. torch full '[2,2]' 1")?;
            let shape: serde_json::Value = serde_json::from_str(shape_text)
                .map_err(|e| format!("shape is not valid JSON: {e}"))?;
            let value_text = args
                .positional
                .get(1)
                .ok_or("full needs a fill value, e.g. torch full '[2,2]' 1")?;
            let value: serde_json::Value = serde_json::from_str(value_text)
                .map_err(|e| format!("fill value is not a number: {e}"))?;
            Ok(serde_json::json!({
                "op": "full",
                "shape": shape,
                "value": value,
                "dtype": args.dtype,
            }))
        }
        "add" => {
            let (a, b) = binary_handles(args, "add")?;
            Ok(serde_json::json!({ "op": "add", "a": a, "b": b }))
        }
        "mm" => {
            let (a, b) = binary_handles(args, "mm")?;
            Ok(serde_json::json!({ "op": "mm", "a": a, "b": b }))
        }
        "mean" => {
            let handle = positional_or_stdin(args, 0, "tensor handle")?;
            Ok(serde_json::json!({ "op": "mean", "handle": handle }))
        }
        "value" => {
            let handle = positional_or_stdin(args, 0, "tensor handle")?;
            Ok(serde_json::json!({ "op": "value", "handle": handle }))
        }
        other => Err(format!("unknown op: {other}")),
    }
}

fn round_trip(socket: &PathBuf, request: &serde_json::Value) -> Result<String, String> {
    let stream = UnixStream::connect(socket)
        .map_err(|e| format!("cannot connect to nutorchd at {}: {e}", socket.display()))?;
    let mut writer = stream
        .try_clone()
        .map_err(|e| format!("socket error: {e}"))?;
    let mut payload = request.to_string();
    payload.push('\n');
    writer
        .write_all(payload.as_bytes())
        .map_err(|e| format!("send failed: {e}"))?;
    writer.flush().map_err(|e| format!("send failed: {e}"))?;

    let mut response = String::new();
    BufReader::new(stream)
        .read_line(&mut response)
        .map_err(|e| format!("receive failed: {e}"))?;
    if response.trim().is_empty() {
        return Err("empty response from daemon".to_string());
    }
    Ok(response)
}

/// Send one request and parse the daemon's reply. Does not spawn.
fn exchange(socket: &PathBuf, request: &serde_json::Value) -> Result<serde_json::Value, String> {
    let response_text = round_trip(socket, request)?;
    let response: serde_json::Value =
        serde_json::from_str(&response_text).map_err(|e| format!("bad response: {e}"))?;
    if response["ok"] == serde_json::Value::Bool(true) {
        Ok(response)
    } else {
        Err(response["error"]
            .as_str()
            .unwrap_or("unknown daemon error")
            .to_string())
    }
}

fn daemon_pid(socket: &PathBuf) -> Result<u64, String> {
    let status = exchange(socket, &serde_json::json!({"op":"status"}))?;
    status["value"]["pid"]
        .as_u64()
        .ok_or_else(|| "status response missing pid".to_string())
}

fn print_status(status: &serde_json::Value) {
    let v = &status["value"];
    let opt = |field: &serde_json::Value, suffix: &str| -> String {
        match field.as_u64() {
            Some(n) => format!("{n}{suffix}"),
            None => "none".to_string(),
        }
    };
    println!("pid: {}", v["pid"]);
    println!("uptime: {}s", v["uptime_secs"]);
    println!("device: {}", v["device"].as_str().unwrap_or("?"));
    println!("ttl: {}", opt(&v["ttl_secs"], "s"));
    println!("idle: {}s", v["idle_secs"]);
    println!("remaining: {}", opt(&v["remaining_secs"], "s"));
    println!("tensors: {}", v["tensors"]);
    println!("memory: ~{} bytes", v["approx_bytes"]);
    println!("socket: {}", v["socket"].as_str().unwrap_or("?"));
    println!("log: {}", v["log"].as_str().unwrap_or("?"));
}

/// `torch daemon <verb>`: positionals only, never stdin (these verbs have no
/// pipeline form). status/ttl/stop never spawn; start/restart do.
fn run_daemon_verb(args: &Args, socket: &PathBuf) -> Result<(), String> {
    let verb = args
        .positional
        .first()
        .ok_or("usage: torch daemon <status|ttl|stop|restart|start>")?;
    match verb.as_str() {
        "status" => {
            if !daemon_alive(socket) {
                return Err(format!("daemon not running (socket: {})", socket.display()));
            }
            let status = exchange(socket, &serde_json::json!({"op":"status"}))?;
            print_status(&status);
            Ok(())
        }
        "ttl" => {
            let duration = args
                .positional
                .get(1)
                .ok_or("usage: torch daemon ttl <duration>  (e.g. 30m, 2h, none)")?;
            if !daemon_alive(socket) {
                return Err(format!("daemon not running (socket: {})", socket.display()));
            }
            let reply = exchange(socket, &serde_json::json!({"op":"set_ttl","ttl":duration}))?;
            match reply["value"]["ttl_secs"].as_u64() {
                Some(secs) => println!("ttl: {secs}s"),
                None => println!("ttl: none"),
            }
            Ok(())
        }
        "stop" => {
            if !daemon_alive(socket) {
                println!("daemon not running (nothing to stop)");
                return Ok(());
            }
            exchange(socket, &serde_json::json!({"op":"shutdown"}))?;
            println!("daemon stopped");
            Ok(())
        }
        "restart" => {
            if daemon_alive(socket) {
                exchange(socket, &serde_json::json!({"op":"shutdown"}))?;
                // The old daemon flushes the shutdown reply BEFORE unlinking;
                // wait for the socket to actually die so the new daemon
                // cannot probe the dying one, yield, and leave zero daemons.
                for _ in 0..60 {
                    if !daemon_alive(socket) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
            ensure_daemon(socket)?;
            println!("daemon restarted (pid {})", daemon_pid(socket)?);
            Ok(())
        }
        "start" => {
            if daemon_alive(socket) {
                println!("already running (pid {})", daemon_pid(socket)?);
            } else {
                ensure_daemon(socket)?;
                println!("started (pid {})", daemon_pid(socket)?);
            }
            Ok(())
        }
        other => Err(format!(
            "unknown daemon verb: {other} (expected status, ttl, stop, restart, or start)"
        )),
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    let socket = args
        .socket
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(default_socket_path);

    if args.op == "daemon" {
        return run_daemon_verb(&args, &socket);
    }

    let request = build_request(&args)?;
    // Auto-start (issue 0004): tensor ops spawn the daemon when it is down.
    if !daemon_alive(&socket) {
        ensure_daemon(&socket)?;
    }
    let response = exchange(&socket, &request)?;
    if let Some(handle) = response["handle"].as_str() {
        println!("{handle}");
    } else {
        println!("{}", response["value"]);
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("torch: {message}");
            ExitCode::FAILURE
        }
    }
}
