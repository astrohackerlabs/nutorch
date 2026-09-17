//! Default-profile behavior. NUTORCH_TEST_BINARY can qualify an installed artifact.
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    path::PathBuf,
    process::{Child, Command, Output, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(30);

struct Shell {
    home: tempfile::TempDir,
}

impl Shell {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir(home.path().join("nushell")).unwrap();
        std::fs::write(home.path().join("nushell/config.nu"), "# isolated\n").unwrap();
        Self { home }
    }

    fn command(&self) -> Command {
        let mut cmd = self.configured_command();
        cmd.arg("--no-config-file");
        cmd
    }

    fn configured_command(&self) -> Command {
        let binary = std::env::var_os("NUTORCH_TEST_BINARY")
            .unwrap_or_else(|| env!("CARGO_BIN_EXE_nutorch").into());
        let mut cmd = Command::new(binary);
        cmd.env("XDG_CONFIG_HOME", self.home.path())
            .env("XDG_CACHE_HOME", self.home.path().join("cache"))
            .env("XDG_DATA_HOME", self.home.path().join("data"));
        if std::env::var_os("NUTORCH_TEST_BINARY").is_some() {
            for name in [
                "DYLD_LIBRARY_PATH",
                "DYLD_FALLBACK_LIBRARY_PATH",
                "LD_LIBRARY_PATH",
            ] {
                cmd.env_remove(name);
            }
        }
        for name in [
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
        ] {
            cmd.env_remove(name);
        }
        cmd
    }

    fn run(&self, code: &str) -> Output {
        output(self.command().args(["-c", code]))
    }

    fn json(&self, code: &str) -> Value {
        let out = self.run(&format!("{code} | to json --raw"));
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).unwrap()
    }
}

// Drain pipes concurrently and kill/reap on timeout, including protocol failures.
struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn output(cmd: &mut Command) -> Output {
    let mut child = Process(
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut stdout = child.0.stdout.take().unwrap();
    let mut stderr = child.0.stderr.take().unwrap();
    let out = thread::spawn(move || {
        let mut b = Vec::new();
        stdout.read_to_end(&mut b).unwrap();
        b
    });
    let err = thread::spawn(move || {
        let mut b = Vec::new();
        stderr.read_to_end(&mut b).unwrap();
        b
    });
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(start.elapsed() < TIMEOUT, "subprocess exceeded {TIMEOUT:?}");
        thread::sleep(Duration::from_millis(10));
    };
    Output {
        status,
        stdout: out.join().unwrap(),
        stderr: err.join().unwrap(),
    }
}

#[test]
fn default_commands_and_sqlite_are_available() {
    let shell = Shell::new();
    let names = shell.json("scope commands | get name");
    for name in [
        "http",
        "http delete",
        "http get",
        "http head",
        "http options",
        "http patch",
        "http pool",
        "http post",
        "http put",
        "port",
        "version check",
        "nutorch sync",
    ] {
        assert!(
            names.as_array().unwrap().contains(&json!(name)),
            "missing {name}"
        );
    }
    assert_eq!(shell.json("let db = ($env.XDG_CONFIG_HOME | path join parity.db); [{a: 42}] | into sqlite $db; open $db | query db 'select a from main'"), json!([{"a":42}]));
}

#[test]
#[cfg(target_os = "macos")]
fn complete_inventory_matches_pinned_default_reference() {
    let shell = Shell::new();
    let baseline: Value =
        serde_json::from_str(include_str!("fixtures/nushell-default-commands.json")).unwrap();
    for (key, flags) in [("without_std", vec!["--no-std-lib"]), ("with_std", vec![])] {
        let out = output(shell.command().args(flags).args(["-c", "scope commands | where type in [built-in keyword] | select name type | sort-by name | to json"]));
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let actual: Vec<Value> = serde_json::from_slice(&out.stdout).unwrap();
        let expected = baseline[key].as_array().unwrap();
        let missing: Vec<_> = expected
            .iter()
            .filter(|row| !actual.contains(row))
            .collect();
        let extra: Vec<_> = actual
            .iter()
            .filter(|row| !expected.contains(row))
            .collect();
        assert!(missing.is_empty(), "{key}: missing {missing:?}");
        assert_eq!(
            extra,
            vec![&json!({"name":"nutorch sync","type":"built-in"})],
            "{key}"
        );
    }
}

#[test]
fn https_rejects_untrusted_local_certificate() {
    let shell = Shell::new();
    let cert = shell.home.path().join("cert.pem");
    let key = shell.home.path().join("key.pem");
    let generated = output(
        Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-days",
                "1",
                "-subj",
                "/CN=localhost",
                "-addext",
                "subjectAltName=DNS:localhost",
                "-out",
            ])
            .arg(&cert)
            .arg("-keyout")
            .arg(&key),
    );
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let tls_log = shell.home.path().join("tls-server.log");
    let mut server = Process(
        Command::new("openssl")
            .args(["s_server", "-accept", &format!("127.0.0.1:{port}"), "-cert"])
            .arg(&cert)
            .arg("-key")
            .arg(key)
            .args(["-www", "-state"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(std::fs::File::create(&tls_log).unwrap())
            .spawn()
            .unwrap(),
    );
    let start = Instant::now();
    while std::net::TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(server.0.try_wait().unwrap().is_none(), "TLS fixture exited");
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "TLS fixture failed to listen"
        );
        thread::sleep(Duration::from_millis(10));
    }
    for pool in ["", "--pool"] {
        let out = shell.run(&format!(
            "http get {pool} --max-time 5sec https://localhost:{port}/"
        ));
        assert!(!out.status.success(), "accepted untrusted certificate");
        let err = String::from_utf8_lossy(&out.stderr).to_lowercase();
        assert!(
            err.contains("certificate")
                || err.contains("unknownissuer")
                || err.contains("nu::shell::io::invalid_data"),
            "not a certificate rejection: {err}"
        );
    }
    // Nu's upstream conversion collapses TLS errors into InvalidData. Confirm
    // both clients reached certificate exchange, then establish the fixture can
    // complete verified TLS with explicit process-local trust in another client.
    thread::sleep(Duration::from_millis(100));
    let log = std::fs::read_to_string(tls_log).unwrap().to_lowercase();
    assert_eq!(
        log.matches("write server certificate verify").count(),
        2,
        "{log}"
    );
    let trusted = output(
        Command::new("openssl")
            .args([
                "s_client",
                "-connect",
                &format!("127.0.0.1:{port}"),
                "-servername",
                "localhost",
                "-verify_hostname",
                "localhost",
                "-verify_return_error",
                "-CAfile",
            ])
            .arg(&cert)
            .arg("-brief")
            .stdin(Stdio::null()),
    );
    assert!(
        trusted.status.success(),
        "{}",
        String::from_utf8_lossy(&trusted.stderr)
    );
    assert!(String::from_utf8_lossy(&trusted.stderr).contains("Verification: OK"));
}

#[test]
#[ignore = "requires public HTTPS services; run explicitly for release qualification"]
fn public_https_hostname_validation_and_version_check() {
    let shell = Shell::new();
    // Independent HTTPS response prevents version check's upstream fallback on
    // network failure from being mistaken for successful update verification.
    let upstream = shell.json(
        "http get --max-time 15sec https://api.github.com/repos/nushell/nushell/releases/latest",
    );
    let tag = upstream["tag_name"]
        .as_str()
        .expect("GitHub release tag")
        .trim_start_matches('v');
    let version = shell.json("version check");
    assert_eq!(version["channel"], "release");
    let embedded = shell.json("version | get version");
    let embedded = embedded.as_str().unwrap();
    let parts = |s: &str| {
        s.split('.')
            .map(|part| part.parse::<u64>().unwrap())
            .collect::<Vec<_>>()
    };
    let newer = parts(tag) > parts(embedded);
    assert_eq!(version["latest"], if newer { tag } else { embedded });
    assert_eq!(version["current"], !newer);
    let identity = output(shell.command().arg("--version"));
    assert!(String::from_utf8_lossy(&identity.stdout).starts_with("nutorch "));
    let mismatch = output(
        Command::new("openssl")
            .args([
                "s_client",
                "-connect",
                "wrong.host.badssl.com:443",
                "-servername",
                "wrong.host.badssl.com",
                "-verify_hostname",
                "wrong.host.badssl.com",
                "-verify_return_error",
            ])
            .stdin(Stdio::null()),
    );
    assert!(!mismatch.status.success());
    assert!(
        String::from_utf8_lossy(&mismatch.stderr).contains("hostname mismatch"),
        "{}",
        String::from_utf8_lossy(&mismatch.stderr)
    );
    for pool in ["", "--pool"] {
        let valid = shell.run(&format!(
            "http get {pool} --max-time 15sec https://example.com | str contains 'Example Domain'"
        ));
        assert!(
            valid.status.success(),
            "{}",
            String::from_utf8_lossy(&valid.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&valid.stdout).trim(), "true");
        let invalid = shell.run(&format!(
            "http get {pool} --max-time 15sec https://wrong.host.badssl.com/"
        ));
        assert!(!invalid.status.success(), "accepted hostname mismatch");
        let err = String::from_utf8_lossy(&invalid.stderr).to_lowercase();
        assert!(
            err.contains("notvalidforname")
                || err.contains("hostname")
                || err.contains("name mismatch")
                || err.contains("nu::shell::io::invalid_data"),
            "not a hostname rejection: {err}"
        );
    }
}

#[test]
fn absent_plugin_is_an_error() {
    let out = Shell::new().run("plugin use nutorch_nonexistent_parity_plugin");
    assert!(!out.status.success(), "plugin use silently succeeded");
    assert!(
        String::from_utf8_lossy(&out.stderr)
            .to_lowercase()
            .contains("plugin")
    );
}

#[test]
fn register_load_and_invoke_real_plugin() {
    let shell = Shell::new();
    let plugin = std::env::var_os("NUTORCH_TEST_PLUGIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let shell = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let forks = [shell.join("../forks"), shell.join("../../../../forks")]
                .into_iter()
                .find(|p| p.join("nushell/Cargo.toml").is_file())
                .expect("pinned forks directory");
            forks.join("nushell/target/debug/nu_plugin_example")
        });
    assert!(
        plugin.is_file(),
        "build the pinned nu_plugin_example first: {}",
        plugin.display()
    );
    let registry = shell.home.path().join("plugins.msgpackz");
    let add = output(
        shell
            .configured_command()
            .arg("--plugin-config")
            .arg(&registry)
            .env("NUTORCH_TEST_PLUGIN", plugin)
            .args(["-c", "plugin add $env.NUTORCH_TEST_PLUGIN"]),
    );
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let invoke = output(
        shell
            .configured_command()
            .arg("--plugin-config")
            .arg(&registry)
            .args([
                "-c",
                "plugin use example; [1 2 3] | example echo | to json --raw",
            ]),
    );
    assert!(
        invoke.status.success(),
        "{}",
        String::from_utf8_lossy(&invoke.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&invoke.stdout).unwrap(),
        json!([1, 2, 3])
    );
}

// Tiny bounded HTTP/1.1 fixture: one connection serves multiple requests so we
// can verify actual pool reuse and reset, not merely the existence of a flag.
fn http_fixture(
    requests: usize,
) -> (
    u16,
    thread::JoinHandle<Vec<(usize, String, String, String)>>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    listener.set_nonblocking(true).unwrap();
    let task = thread::spawn(move || {
        let start = Instant::now();
        let mut seen = Vec::new();
        let mut connection = 0;
        while seen.len() < requests {
            assert!(
                start.elapsed() < TIMEOUT,
                "HTTP fixture timed out: {seen:?}"
            );
            let (stream, _) = match listener.accept() {
                Ok(pair) => pair,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => panic!("{e}"),
            };
            connection += 1;
            // Darwin may inherit O_NONBLOCK from the listening socket.
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut reader = BufReader::new(stream);
            loop {
                let mut first = String::new();
                if reader.read_line(&mut first).unwrap_or(0) == 0 {
                    break;
                }
                let method = first.split_whitespace().next().unwrap().to_string();
                let mut headers = String::new();
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    if let Some((key, value)) = line.split_once(':') {
                        if key.eq_ignore_ascii_case("content-length") {
                            length = value.trim().parse().unwrap();
                        }
                    }
                    headers.push_str(&line.to_lowercase());
                }
                let mut body = vec![0; length];
                reader.read_exact(&mut body).unwrap();
                seen.push((
                    connection,
                    method.clone(),
                    headers,
                    String::from_utf8(body).unwrap(),
                ));
                let body = if method == "HEAD" {
                    ""
                } else {
                    "{\"ok\":true}"
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nConnection: keep-alive\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Parity: yes\r\n\r\n{}",
                    body.len(),
                    body
                );
                reader.get_mut().write_all(response.as_bytes()).unwrap();
                reader.get_mut().flush().unwrap();
                if seen.len() == requests {
                    break;
                }
            }
        }
        seen
    });
    (port, task)
}

#[test]
fn http_methods_send_headers_and_bodies() {
    let (port, task) = http_fixture(7);
    let shell = Shell::new();
    for method in ["get", "head", "options", "post", "put", "patch", "delete"] {
        let data = if ["post", "put", "patch"].contains(&method) {
            " 'payload'"
        } else {
            ""
        };
        let value = shell.json(&format!("http {method} --max-time 5sec --headers {{X-Parity: request}} http://127.0.0.1:{port}/{data}"));
        if ["head", "options"].contains(&method) {
            assert!(value.to_string().contains("yes"));
        } else {
            assert_eq!(value, json!({"ok":true}));
        }
    }
    let seen = task.join().unwrap();
    assert_eq!(
        seen.iter().map(|r| r.1.as_str()).collect::<Vec<_>>(),
        ["GET", "HEAD", "OPTIONS", "POST", "PUT", "PATCH", "DELETE"]
    );
    for (_, method, headers, body) in seen {
        assert!(headers.contains("x-parity: request"));
        assert_eq!(
            body,
            if ["POST", "PUT", "PATCH"].contains(&method.as_str()) {
                "payload"
            } else {
                ""
            }
        );
    }
}

#[test]
fn http_pool_reuses_then_resets_connections() {
    let (port, task) = http_fixture(3);
    let code = format!(
        "http get --pool --max-time 5sec http://127.0.0.1:{port}/ | ignore; http get --pool --max-time 5sec http://127.0.0.1:{port}/ | ignore; http pool; http get --pool --max-time 5sec http://127.0.0.1:{port}/"
    );
    assert_eq!(Shell::new().json(&code), json!({"ok":true}));
    let seen = task.join().unwrap();
    assert_eq!(seen.iter().map(|r| r.0).collect::<Vec<_>>(), [1, 1, 2]);
}

#[test]
fn port_rejects_occupied_candidate() {
    let shell = Shell::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let occupied = shell.run(&format!("port {port} {port}"));
    assert!(!occupied.status.success());
    drop(listener);
    assert_eq!(shell.json(&format!("port {port} {port}")), json!(port));
    let selected = shell.json("port").as_u64().unwrap();
    TcpListener::bind(("127.0.0.1", selected as u16)).unwrap();
}

fn protocol(lsp: bool) {
    let shell = Shell::new();
    let mut child = Process(
        shell
            .command()
            .arg(if lsp { "--lsp" } else { "--mcp" })
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let mut input = child.0.stdin.take().unwrap();
    let stdout = child.0.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                break;
            }
            let bytes = if lsp {
                let length: usize = line
                    .trim()
                    .strip_prefix("Content-Length: ")
                    .unwrap()
                    .parse()
                    .unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                let mut bytes = vec![0; length];
                reader.read_exact(&mut bytes).unwrap();
                bytes
            } else {
                line.into_bytes()
            };
            if tx
                .send(serde_json::from_slice::<Value>(&bytes).unwrap())
                .is_err()
            {
                break;
            }
        }
    });
    let mut send = |v: Value| {
        let msg = v.to_string();
        if lsp {
            write!(input, "Content-Length: {}\r\n\r\n{msg}", msg.len()).unwrap();
        } else {
            writeln!(input, "{msg}").unwrap();
        }
        input.flush().unwrap();
    };
    let params = if lsp {
        json!({"processId":null,"capabilities":{},"rootUri":null})
    } else {
        json!({"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"nutorch-parity","version":"1"}})
    };
    send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":params}));
    let first = rx.recv_timeout(TIMEOUT).expect("initialize response");
    assert_eq!(first["id"], 1);
    assert!(first.get("error").is_none(), "{first}");
    assert!(first["result"]["capabilities"].is_object());
    if !lsp {
        send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
        send(
            json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"evaluate","arguments":{"input":"1 + 2"}}}),
        );
        let response = rx.recv_timeout(TIMEOUT).expect("MCP evaluate response");
        assert_eq!(response["id"], 2);
        assert!(response.get("error").is_none(), "{response}");
        assert_ne!(response["result"]["isError"], true, "{response}");
        assert!(
            response["result"]["content"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["text"].as_str().is_some_and(|s| s.contains('3'))),
            "{response}"
        );
    }
}

#[test]
fn mcp_initializes_and_evaluates() {
    protocol(false);
}

#[test]
fn lsp_still_initializes() {
    protocol(true);
}
