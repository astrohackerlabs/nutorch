//! nutorchd: the Nutorch v2 daemon (PoC, issue 0002).
//!
//! Owns the tensor registry and the LibTorch context; serves newline-delimited
//! JSON requests over a Unix socket. One connection at a time (PoC).
//!
//! Socket handling (issue 0004): probe-before-bind — a live daemon is never
//! displaced; a newcomer finding a live daemon exits 0 quietly (see
//! issues/0004-daemon-lifecycle/01-daemon-side-lifecycle.md).

mod convert;
mod lifecycle;
mod protocol;
mod registry;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use lifecycle::Lifecycle;
use protocol::{Request, Response};
use registry::Registry;
use tch::{Device, Kind, Tensor};

const DEFAULT_TTL: Duration = Duration::from_secs(3600);

fn default_socket_path() -> PathBuf {
    match std::env::var_os("TMPDIR") {
        Some(tmp) => PathBuf::from(tmp).join("nutorchd.sock"),
        None => PathBuf::from("/tmp/nutorchd.sock"),
    }
}

struct DaemonArgs {
    socket: PathBuf,
    ttl: Option<Duration>,
}

/// `--ttl` flag wins over `NUTORCHD_TTL` env, which wins over the 1h default.
fn parse_daemon_args() -> Result<DaemonArgs, String> {
    let mut socket = None;
    let mut ttl_text: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--socket" => socket = Some(args.next().ok_or("--socket needs a value")?),
            "--ttl" => ttl_text = Some(args.next().ok_or("--ttl needs a value")?),
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    let ttl = match ttl_text.or_else(|| std::env::var("NUTORCHD_TTL").ok()) {
        Some(text) => lifecycle::parse_ttl(&text)?,
        None => Some(DEFAULT_TTL),
    };
    Ok(DaemonArgs {
        socket: socket
            .map(PathBuf::from)
            .unwrap_or_else(default_socket_path),
        ttl,
    })
}

/// The conventional log path: the socket path with its extension replaced
/// by `.log` (nutorchd.sock -> nutorchd.log). The
/// spawner (client auto-start) redirects daemon output here; for a manually
/// started daemon the file may not exist — it is a convention report.
fn log_path_for(socket: &Path) -> PathBuf {
    socket.with_extension("log")
}

/// Probe-before-bind (issue 0004): never steal a live daemon's socket.
/// Returns Ok(None) when another daemon is already serving the path (the
/// caller should exit 0 quietly). Known residual: a simultaneous-start
/// TOCTOU window remains (both probe before either binds); see the
/// experiment doc.
fn probe_and_bind(socket: &Path) -> std::io::Result<Option<UnixListener>> {
    if UnixStream::connect(socket).is_ok() {
        return Ok(None);
    }
    // Refused or missing: any file present is stale — remove and bind.
    let _ = std::fs::remove_file(socket);
    UnixListener::bind(socket).map(Some)
}

/// nutorchd is GPU-only (issue 0003): Mac-only for now, so the GPU is MPS,
/// and the daemon refuses to start without it.
fn require_mps() -> Result<(), String> {
    if tch::utils::has_mps() {
        Ok(())
    } else {
        Err("nutorchd requires an Apple-silicon Mac with MPS (GPU-only by design)".to_string())
    }
}

/// Reject a request that still carries the removed `device` option (issue
/// 0003) with an explanatory error, before deserializing into `Request`.
/// (serde's deny_unknown_fields does not work on internally tagged enums, so
/// this special case is checked explicitly; other unknown fields stay
/// ignored.)
fn parse_request(line: &str) -> Result<Request, String> {
    let raw: serde_json::Value =
        serde_json::from_str(line).map_err(|e| format!("bad request: {e}"))?;
    if raw.get("device").is_some() {
        return Err(
            "the device option was removed (issue 0003): tensors always live on the GPU (mps)"
                .to_string(),
        );
    }
    serde_json::from_value(raw).map_err(|e| format!("bad request: {e}"))
}

/// Look up two operand handles. Every registry tensor lives on MPS (issue
/// 0003), so device agreement is an invariant, not a user error — asserted
/// in debug builds.
fn binary_operands<'r>(
    registry: &'r Registry,
    a: &str,
    b: &str,
) -> Result<(&'r Tensor, &'r Tensor), String> {
    let ta = registry
        .get(a)
        .ok_or_else(|| format!("unknown handle: {a}"))?;
    let tb = registry
        .get(b)
        .ok_or_else(|| format!("unknown handle: {b}"))?;
    debug_assert_eq!(
        ta.device(),
        tb.device(),
        "registry invariant violated: all tensors live on MPS"
    );
    Ok((ta, tb))
}

/// Dispatch one request. Returns the response plus a shutdown flag: when
/// true, the serve loop must write and flush the response FIRST, then unlink
/// the socket and exit (graceful `shutdown` op ordering).
///
/// Tensor ops reset the idle clock; `status`/`set_ttl`/`shutdown` do not
/// (observing or configuring the daemon must not immortalize it).
fn handle_request(
    registry: &mut Registry,
    lifecycle: &Mutex<Lifecycle>,
    socket: &Path,
    request: Request,
) -> (Response, bool) {
    if matches!(
        request,
        Request::Tensor { .. }
            | Request::Full { .. }
            | Request::Add { .. }
            | Request::Mm { .. }
            | Request::Mean { .. }
            | Request::Value { .. }
    ) {
        lifecycle.lock().unwrap().touch();
    }

    let response = match request {
        Request::Status => {
            let state = lifecycle.lock().unwrap();
            return (
                Response::value(serde_json::json!({
                    "pid": std::process::id(),
                    "uptime_secs": state.uptime_secs(),
                    "device": "mps",
                    "ttl_secs": state.ttl_secs(),
                    "idle_secs": state.idle_secs(),
                    "remaining_secs": state.remaining_secs(),
                    "tensors": registry.len(),
                    "approx_bytes": registry.approx_bytes(),
                    "socket": socket.display().to_string(),
                    "log": log_path_for(socket).display().to_string(),
                })),
                false,
            );
        }
        Request::SetTtl { ttl } => match lifecycle::parse_ttl(&ttl) {
            Ok(parsed) => {
                let mut state = lifecycle.lock().unwrap();
                state.set_ttl(parsed);
                return (
                    Response::value(serde_json::json!({ "ttl_secs": state.ttl_secs() })),
                    false,
                );
            }
            Err(e) => return (Response::error(e), false),
        },
        Request::Shutdown => {
            return (Response::value(serde_json::json!("shutting down")), true);
        }
        other => dispatch_tensor_op(registry, other),
    };
    (response, false)
}

fn dispatch_tensor_op(registry: &mut Registry, request: Request) -> Response {
    match request {
        Request::Tensor { data, dtype } => {
            let kind = match convert::parse_kind(dtype.as_deref()) {
                Ok(k) => k,
                Err(e) => return Response::error(e),
            };
            match convert::json_to_tensor(&data, kind, Device::Mps) {
                Ok(tensor) => Response::handle(registry.insert(tensor)),
                Err(e) => Response::error(e),
            }
        }
        Request::Full {
            shape,
            value,
            dtype,
        } => {
            // Rust-side shape validation, ported from v1 command_full.rs:
            // non-empty, every dimension >= 1.
            if shape.is_empty() {
                return Response::error("shape cannot be empty");
            }
            if let Some(bad) = shape.iter().find(|d| **d < 1) {
                return Response::error(format!(
                    "invalid shape: every dimension must be >= 1, got {bad}"
                ));
            }
            let kind = match convert::parse_kind(dtype.as_deref()) {
                Ok(k) => k,
                Err(e) => return Response::error(e),
            };
            let result = if let Some(i) = value.as_i64() {
                Tensor::f_full(&shape, i, (kind, Device::Mps))
            } else if let Some(f) = value.as_f64() {
                Tensor::f_full(&shape, f, (kind, Device::Mps))
            } else {
                return Response::error(format!("fill value must be a number, got {value}"));
            };
            match result {
                Ok(tensor) => Response::handle(registry.insert(tensor)),
                Err(e) => Response::error(convert::tch_error(e)),
            }
        }
        Request::Add { a, b } => {
            let (ta, tb) = match binary_operands(registry, &a, &b) {
                Ok(pair) => pair,
                Err(e) => return Response::error(e),
            };
            match ta.f_add(tb) {
                Ok(tensor) => Response::handle(registry.insert(tensor)),
                Err(e) => Response::error(convert::tch_error(e)),
            }
        }
        Request::Mm { a, b } => {
            let (ta, tb) = match binary_operands(registry, &a, &b) {
                Ok(pair) => pair,
                Err(e) => return Response::error(e),
            };
            // Rust-side validation ported from v1 command_mm.rs:117-140:
            // both rank-2, inner dimensions equal.
            let (sa, sb) = (ta.size(), tb.size());
            if sa.len() != 2 || sb.len() != 2 {
                return Response::error(format!(
                    "mm requires two 2-D tensors, got shapes {sa:?} and {sb:?}"
                ));
            }
            if sa[1] != sb[0] {
                return Response::error(format!(
                    "mm shape mismatch: inner dimensions must match, got {sa:?} and {sb:?}"
                ));
            }
            match ta.f_mm(tb) {
                Ok(tensor) => Response::handle(registry.insert(tensor)),
                Err(e) => Response::error(convert::tch_error(e)),
            }
        }
        Request::Mean { handle } => match registry.get(&handle) {
            // v1 fidelity: mean dtype defaults to float32 regardless of the
            // input kind (v1 command_mean.rs:133,152, lib.rs:197); also keeps
            // MPS happy (no float64).
            Some(tensor) => match tensor.f_mean(Kind::Float) {
                Ok(tensor) => Response::handle(registry.insert(tensor)),
                Err(e) => Response::error(convert::tch_error(e)),
            },
            None => Response::error(format!("unknown handle: {handle}")),
        },
        Request::Value { handle } => match registry.get(&handle) {
            Some(tensor) => {
                let cpu = match tensor.f_to_device(Device::Cpu) {
                    Ok(t) => t,
                    Err(e) => return Response::error(convert::tch_error(e)),
                };
                match convert::tensor_to_json(&cpu) {
                    Ok(value) => Response::value(value),
                    Err(e) => Response::error(e),
                }
            }
            None => Response::error(format!("unknown handle: {handle}")),
        },
        Request::Status | Request::SetTtl { .. } | Request::Shutdown => {
            Response::error("internal: lifecycle op routed to tensor dispatch")
        }
    }
}

fn serve_connection(
    registry: &mut Registry,
    lifecycle: &Mutex<Lifecycle>,
    socket: &Path,
    stream: UnixStream,
) -> std::io::Result<()> {
    let mut writer = stream.try_clone()?;
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let (response, shutdown) = match parse_request(&line) {
            Ok(request) => handle_request(registry, lifecycle, socket, request),
            Err(e) => (Response::error(e), false),
        };
        let mut payload = serde_json::to_string(&response).expect("response serializes");
        payload.push('\n');
        writer.write_all(payload.as_bytes())?;
        writer.flush()?;
        if shutdown {
            // Graceful: the response is flushed; now clean up and exit.
            println!("shutdown requested; exiting");
            let _ = std::fs::remove_file(socket);
            std::process::exit(0);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request(json: serde_json::Value) -> Request {
        serde_json::from_value(json).expect("valid request")
    }

    fn test_socket() -> &'static Path {
        Path::new("/tmp/nutorchd-unit-test.sock")
    }

    /// Old-style dispatch with a throwaway lifecycle (default ttl).
    fn run(registry: &mut Registry, req: Request) -> Response {
        let lifecycle = Mutex::new(Lifecycle::new(Some(DEFAULT_TTL)));
        let (response, shutdown) = handle_request(registry, &lifecycle, test_socket(), req);
        assert!(!shutdown, "tensor ops never request shutdown");
        response
    }

    /// Dispatch against a shared lifecycle (for activity-semantics tests).
    fn run_with(
        registry: &mut Registry,
        lifecycle: &Mutex<Lifecycle>,
        req: Request,
    ) -> (Response, bool) {
        handle_request(registry, lifecycle, test_socket(), req)
    }

    fn expect_handle(response: Response) -> String {
        match response {
            Response::Handle { handle, .. } => handle,
            other => panic!("expected handle, got {other:?}"),
        }
    }

    fn expect_value(response: Response) -> serde_json::Value {
        match response {
            Response::Value { value, .. } => value,
            other => panic!("expected value, got {other:?}"),
        }
    }

    fn expect_error(response: Response) -> String {
        match response {
            Response::Error { error, .. } => error,
            other => panic!("expected error, got {other:?}"),
        }
    }

    fn value_of(registry: &mut Registry, handle: &str) -> serde_json::Value {
        expect_value(run(
            registry,
            request(json!({"op":"value","handle":handle})),
        ))
    }

    #[test]
    fn full_round_trips_exactly() {
        let mut registry = Registry::new();
        let h = expect_handle(run(
            &mut registry,
            request(json!({"op":"full","shape":[2,2],"value":1})),
        ));
        assert_eq!(value_of(&mut registry, &h), json!([[1.0, 1.0], [1.0, 1.0]]));
    }

    #[test]
    fn add_is_exact() {
        let mut registry = Registry::new();
        let a = expect_handle(run(
            &mut registry,
            request(json!({"op":"tensor","data":[1,2,3]})),
        ));
        let b = expect_handle(run(
            &mut registry,
            request(json!({"op":"tensor","data":[4,5,6]})),
        ));
        let sum = expect_handle(run(&mut registry, request(json!({"op":"add","a":a,"b":b}))));
        assert_eq!(value_of(&mut registry, &sum), json!([5.0, 7.0, 9.0]));
    }

    #[test]
    fn mm_of_ones_is_exact_and_mean_folds_it() {
        let mut registry = Registry::new();
        let a = expect_handle(run(
            &mut registry,
            request(json!({"op":"full","shape":[2,2],"value":1})),
        ));
        let b = expect_handle(run(
            &mut registry,
            request(json!({"op":"full","shape":[2,2],"value":1})),
        ));
        let product = expect_handle(run(&mut registry, request(json!({"op":"mm","a":a,"b":b}))));
        assert_eq!(
            value_of(&mut registry, &product),
            json!([[2.0, 2.0], [2.0, 2.0]])
        );
        let mean = expect_handle(run(
            &mut registry,
            request(json!({"op":"mean","handle":product})),
        ));
        assert_eq!(value_of(&mut registry, &mean), json!(2.0));
    }

    #[test]
    fn mm_rejects_mismatched_shapes_naming_them() {
        let mut registry = Registry::new();
        let a = expect_handle(run(
            &mut registry,
            request(json!({"op":"full","shape":[2,3],"value":1})),
        ));
        let b = expect_handle(run(
            &mut registry,
            request(json!({"op":"full","shape":[2,3],"value":1})),
        ));
        let error = expect_error(run(&mut registry, request(json!({"op":"mm","a":a,"b":b}))));
        assert!(
            error.contains("[2, 3]"),
            "error should name shapes: {error}"
        );
    }

    #[test]
    fn mm_rejects_non_2d() {
        let mut registry = Registry::new();
        let a = expect_handle(run(
            &mut registry,
            request(json!({"op":"tensor","data":[1,2,3]})),
        ));
        let error = expect_error(run(
            &mut registry,
            request(json!({"op":"mm","a":a.clone(),"b":a})),
        ));
        assert!(error.contains("2-D"), "error should mention rank: {error}");
    }

    #[test]
    fn add_rejects_unknown_handle() {
        let mut registry = Registry::new();
        let a = expect_handle(run(
            &mut registry,
            request(json!({"op":"tensor","data":[1]})),
        ));
        let error = expect_error(run(
            &mut registry,
            request(json!({"op":"add","a":a,"b":"nope"})),
        ));
        assert!(error.contains("unknown handle"), "{error}");
    }

    #[test]
    fn require_mps_holds_on_this_machine() {
        assert!(require_mps().is_ok());
    }

    #[test]
    fn created_tensors_live_on_mps() {
        let mut registry = Registry::new();
        let h = expect_handle(run(
            &mut registry,
            request(json!({"op":"tensor","data":[1,2,3]})),
        ));
        assert_eq!(registry.get(&h).unwrap().device(), Device::Mps);
        let f = expect_handle(run(
            &mut registry,
            request(json!({"op":"full","shape":[2,2],"value":1})),
        ));
        assert_eq!(registry.get(&f).unwrap().device(), Device::Mps);
    }

    #[test]
    fn device_field_is_rejected_with_removal_message() {
        let error = parse_request(r#"{"op":"tensor","data":[1],"device":"cpu"}"#)
            .expect_err("device field must be rejected");
        assert!(error.contains("device option was removed"), "{error}");
        // The same line without the field parses fine.
        assert!(parse_request(r#"{"op":"tensor","data":[1]}"#).is_ok());
    }

    #[test]
    fn status_reports_shape_counts_and_conventions() {
        let mut registry = Registry::new();
        let _ = expect_handle(run(
            &mut registry,
            request(json!({"op":"tensor","data":[1,2,3]})),
        ));
        let lifecycle = Mutex::new(Lifecycle::new(Some(DEFAULT_TTL)));
        let (response, shutdown) =
            run_with(&mut registry, &lifecycle, request(json!({"op":"status"})));
        assert!(!shutdown);
        let status = expect_value(response);
        assert_eq!(status["pid"], std::process::id());
        assert_eq!(status["device"], "mps");
        assert_eq!(status["ttl_secs"], 3600);
        assert_eq!(status["tensors"], 1);
        // three float32 elements = 12 bytes
        assert_eq!(status["approx_bytes"], 12);
        assert!(status["log"].as_str().unwrap().ends_with(".log"));
        assert!(status["remaining_secs"].as_u64().unwrap() > 3590);
    }

    #[test]
    fn set_ttl_changes_remaining_and_rejects_garbage() {
        let mut registry = Registry::new();
        let lifecycle = Mutex::new(Lifecycle::new(Some(DEFAULT_TTL)));
        let (response, _) = run_with(
            &mut registry,
            &lifecycle,
            request(json!({"op":"set_ttl","ttl":"2m"})),
        );
        assert_eq!(expect_value(response)["ttl_secs"], 120);
        let (response, _) = run_with(&mut registry, &lifecycle, request(json!({"op":"status"})));
        assert!(expect_value(response)["remaining_secs"].as_u64().unwrap() <= 120);

        let (response, _) = run_with(
            &mut registry,
            &lifecycle,
            request(json!({"op":"set_ttl","ttl":"none"})),
        );
        assert_eq!(expect_value(response)["ttl_secs"], serde_json::Value::Null);

        let (response, _) = run_with(
            &mut registry,
            &lifecycle,
            request(json!({"op":"set_ttl","ttl":"bogus"})),
        );
        assert!(expect_error(response).contains("invalid ttl"));
    }

    #[test]
    fn tensor_ops_reset_idle_but_status_does_not() {
        let mut registry = Registry::new();
        let lifecycle = Mutex::new(Lifecycle::new(Some(DEFAULT_TTL)));
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let (response, _) = run_with(&mut registry, &lifecycle, request(json!({"op":"status"})));
        assert!(
            expect_value(response)["idle_secs"].as_u64().unwrap() >= 1,
            "status must not reset the idle clock"
        );
        let (_, _) = run_with(
            &mut registry,
            &lifecycle,
            request(json!({"op":"tensor","data":[1]})),
        );
        let (response, _) = run_with(&mut registry, &lifecycle, request(json!({"op":"status"})));
        assert_eq!(
            expect_value(response)["idle_secs"],
            0,
            "a tensor op must reset the idle clock"
        );
    }

    #[test]
    fn shutdown_op_replies_ok_and_raises_the_flag() {
        let mut registry = Registry::new();
        let lifecycle = Mutex::new(Lifecycle::new(Some(DEFAULT_TTL)));
        let (response, shutdown) =
            run_with(&mut registry, &lifecycle, request(json!({"op":"shutdown"})));
        assert!(shutdown);
        assert_eq!(expect_value(response), json!("shutting down"));
    }

    #[test]
    fn full_rejects_bad_shapes() {
        let mut registry = Registry::new();
        let empty = expect_error(run(
            &mut registry,
            request(json!({"op":"full","shape":[],"value":1})),
        ));
        assert!(empty.contains("empty"), "{empty}");
        let zero = expect_error(run(
            &mut registry,
            request(json!({"op":"full","shape":[2,0],"value":1})),
        ));
        assert!(zero.contains(">= 1"), "{zero}");
    }
}

fn main() -> std::io::Result<()> {
    if let Err(message) = require_mps() {
        eprintln!("{message}");
        std::process::exit(1);
    }

    let args = match parse_daemon_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("nutorchd: {message}");
            std::process::exit(2);
        }
    };

    let listener = match probe_and_bind(&args.socket)? {
        Some(listener) => listener,
        None => {
            println!(
                "nutorchd already running on {}; exiting",
                args.socket.display()
            );
            return Ok(());
        }
    };

    println!("nutorchd");
    println!("pid: {}", std::process::id());
    println!("socket: {}", args.socket.display());
    println!("device: mps");
    match args.ttl {
        Some(ttl) => println!("ttl: {}s (idle)", ttl.as_secs()),
        None => println!("ttl: none"),
    }

    let lifecycle = Arc::new(Mutex::new(Lifecycle::new(args.ttl)));

    // Expiry watcher: wakes ~1x/second; on idle expiry, cleans up and exits.
    {
        let lifecycle = Arc::clone(&lifecycle);
        let socket = args.socket.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            if lifecycle.lock().unwrap().expired() {
                println!("idle ttl expired; exiting");
                let _ = std::fs::remove_file(&socket);
                std::process::exit(0);
            }
        });
    }

    // Signal handler: SIGTERM/SIGINT unlink the socket and exit cleanly
    // (fixes the stranded-socket debt from issue 0002).
    {
        let socket = args.socket.clone();
        let mut signals = signal_hook::iterator::Signals::new([
            signal_hook::consts::SIGTERM,
            signal_hook::consts::SIGINT,
        ])?;
        std::thread::spawn(move || {
            if signals.forever().next().is_some() {
                println!("signal received; exiting");
                let _ = std::fs::remove_file(&socket);
                std::process::exit(0);
            }
        });
    }

    let mut registry = Registry::new();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = serve_connection(&mut registry, &lifecycle, &args.socket, stream) {
                    eprintln!("connection error: {e}");
                }
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
    Ok(())
}
