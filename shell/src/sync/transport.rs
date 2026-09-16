use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{HashSet, VecDeque},
    fs::{self, DirBuilder},
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::{
            fs::{DirBuilderExt, MetadataExt, PermissionsExt},
            net::{UnixListener, UnixStream},
        },
    },
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub const SOCKET: &str = "NUTORCH_SYNC_SOCKET";
pub const TOKEN: &str = "NUTORCH_SYNC_TOKEN";
pub const MAX_FRAME: usize = 1024 * 1024;
const MAX_ENTRIES: usize = 4096;
const MAX_VALUE: usize = 64 * 1024;
const MAX_QUEUE: usize = 32;
const MAX_QUEUED_BYTES: usize = 8 * MAX_FRAME;
pub type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone)]
pub struct Endpoint {
    pub path: PathBuf,
    pub token: String,
}

impl Endpoint {
    pub fn inherited() -> Result<Option<Self>> {
        match (std::env::var_os(SOCKET), std::env::var_os(TOKEN)) {
            (None, None) => Ok(None),
            (Some(path), Some(token)) => {
                let token = token
                    .into_string()
                    .map_err(|_| "invalid inherited sync token")?;
                let endpoint = Self {
                    path: path.into(),
                    token,
                };
                endpoint.validate()?;
                Ok(Some(endpoint))
            }
            _ => Err("incomplete inherited sync connection"),
        }
    }

    fn validate(&self) -> Result<()> {
        if !self.path.is_absolute() || socket2::SockAddr::unix(&self.path).is_err() {
            return Err("invalid or overlong sync socket address");
        }
        if self.token.len() != 64 || !self.token.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid inherited sync token");
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u8,
    pub token: String,
    pub env: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub version: u8,
    pub status: String,
    pub sequence: u64,
    pub accepted: usize,
}

pub struct Pending {
    pub sequence: u64,
    pub env: Vec<(String, String)>,
}

#[derive(Default)]
struct Queue {
    entries: VecDeque<Pending>,
    bytes: usize,
    sequence: u64,
}

pub struct Server {
    pub endpoint: Endpoint,
    queue: Arc<Mutex<Queue>>,
    stop: Arc<AtomicBool>,
    thread: Mutex<Option<JoinHandle<()>>>,
    owner_pid: u32,
}

pub fn reserved(name: &str) -> bool {
    // Pinned Nushell uses ASCII-insensitive environment lookup on every OS.
    // Apply the same rule to protection and duplicate detection.
    let uppercase = name.to_ascii_uppercase();
    let name = uppercase.as_str();
    matches!(
        name,
        "PWD"
            | "OLDPWD"
            | "FILE_PWD"
            | "CURRENT_FILE"
            | "SHLVL"
            | "_"
            | "CONFIG"
            | "ENV_CONVERSIONS"
            | "NU_LIB_DIRS"
            | "NU_PLUGIN_DIRS"
            | "NU_LOG_LEVEL"
            | "LAST_EXIT_CODE"
            | "CMD_DURATION_MS"
            | "NUTORCH_MODE"
    ) || [
        "NUTORCH_SYNC_",
        "PROMPT_",
        "TRANSIENT_PROMPT_",
        "BASH_FUNC_",
        "TERMSURF_",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
}

pub fn validate_entries(entries: &[(String, String)]) -> Result<()> {
    if entries.len() > MAX_ENTRIES {
        return Err("too many environment entries");
    }
    let mut names = HashSet::new();
    for (name, value) in entries {
        if name.is_empty() || name.contains(['\0', '=']) || value.contains('\0') {
            return Err("invalid environment entry");
        }
        if value.len() > MAX_VALUE {
            return Err("environment value exceeds limit");
        }
        if !names.insert(name.to_ascii_uppercase()) {
            return Err("duplicate environment name");
        }
    }
    Ok(())
}

fn random_hex(bytes: usize) -> Result<String> {
    let mut data = vec![0; bytes];
    getrandom::fill(&mut data).map_err(|_| "cannot obtain sync randomness")?;
    Ok(data.iter().map(|b| format!("{b:02x}")).collect())
}

fn private_dir(path: &Path, create: bool) -> Result<()> {
    if create {
        match DirBuilder::new().mode(0o700).create(path) {
            Ok(()) => (),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(_) => return Err("cannot create private sync directory"),
        }
    }
    let meta = fs::symlink_metadata(path).map_err(|_| "sync directory is unavailable")?;
    // SAFETY: getuid has no preconditions.
    if !meta.is_dir() || meta.uid() != unsafe { libc::getuid() } || meta.mode() & 0o777 != 0o700 {
        return Err("sync directory must be owned by this user with mode 0700 and not a symlink");
    }
    Ok(())
}

pub fn data_home(data: Option<&Path>, home: Option<&Path>) -> Result<PathBuf> {
    if let Some(base) = data.filter(|p| p.is_absolute()) {
        return Ok(base.to_path_buf());
    }
    let home = home
        .filter(|p| p.is_absolute())
        .ok_or("set an absolute HOME or XDG_DATA_HOME for sync socket storage")?;
    Ok(home.join(".local/share"))
}

fn socket_dir(base: &Path, id: &str) -> Result<PathBuf> {
    if !base.is_absolute() {
        return Err("XDG data directory must be absolute");
    }
    let parent = base.join("astrohacker");
    let dir = parent.join("nutorch");
    socket2::SockAddr::unix(dir.join(format!("{id}.sock")))
        .map_err(|_| "XDG data socket path is too long; set a shorter absolute XDG_DATA_HOME")?;
    fs::create_dir_all(base)
        .map_err(|_| "cannot create XDG data directory; check HOME or XDG_DATA_HOME")?;
    match DirBuilder::new().mode(0o700).create(&parent) {
        Ok(()) => (),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(_) => return Err("cannot create Astrohacker data directory"),
    }
    let meta =
        fs::symlink_metadata(&parent).map_err(|_| "Astrohacker data directory is unavailable")?;
    // Shared Astrohacker data may be readable; it must not be writable by others.
    if !meta.is_dir() || meta.uid() != unsafe { libc::getuid() } || meta.mode() & 0o022 != 0 {
        return Err(
            "Astrohacker data directory must be user-owned, not a symlink, and not writable by others",
        );
    }
    private_dir(&dir, true)?;
    Ok(dir)
}

fn remaining(deadline: Instant) -> Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or("sync I/O deadline exceeded")
}

fn read_exact(stream: &mut UnixStream, mut data: &mut [u8], deadline: Instant) -> Result<()> {
    stream
        .set_nonblocking(true)
        .map_err(|_| "cannot configure sync I/O")?;
    while !data.is_empty() {
        wait_ready(stream, libc::POLLIN, deadline)?;
        match stream.read(data) {
            Ok(0) => return Err("incomplete sync frame"),
            Ok(n) => {
                data = &mut data[n..];
            }
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(_) => return Err("sync read failed or timed out"),
        }
    }
    Ok(())
}

fn wait_ready(stream: &UnixStream, events: libc::c_short, deadline: Instant) -> Result<()> {
    loop {
        let millis = remaining(deadline)?
            .as_millis()
            .saturating_add(1)
            .min(i32::MAX as u128) as i32;
        let mut fd = libc::pollfd {
            fd: stream.as_raw_fd(),
            events,
            revents: 0,
        };
        // SAFETY: one initialized pollfd, borrowed live socket, bounded timeout.
        let result = unsafe { libc::poll(&mut fd, 1, millis) };
        if result > 0 {
            return Ok(());
        }
        if result == 0 {
            return Err("sync I/O deadline exceeded");
        }
        if std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted {
            return Err("sync I/O polling failed");
        }
    }
}

fn read_frame<T: DeserializeOwned>(
    stream: &mut UnixStream,
    deadline: Instant,
) -> Result<(T, usize)> {
    let mut size = [0; 4];
    read_exact(stream, &mut size, deadline)?;
    let size = u32::from_be_bytes(size) as usize;
    if size == 0 || size > MAX_FRAME {
        return Err("invalid sync frame length");
    }
    let mut data = vec![0; size];
    read_exact(stream, &mut data, deadline)?;
    let value = serde_json::from_slice(&data).map_err(|_| "invalid sync message")?;
    Ok((value, size))
}

fn write_frame<T: Serialize>(stream: &mut UnixStream, value: &T, deadline: Instant) -> Result<()> {
    struct Limited(Vec<u8>);
    impl Write for Limited {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.0.len() + bytes.len() > MAX_FRAME {
                return Err(std::io::Error::other("frame limit"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut data = Limited(Vec::new());
    serde_json::to_writer(&mut data, value)
        .map_err(|_| "sync frame exceeds limit or cannot be encoded")?;
    let data = data.0;
    stream
        .set_nonblocking(true)
        .map_err(|_| "cannot configure sync I/O")?;
    for mut bytes in [&(data.len() as u32).to_be_bytes()[..], &data[..]] {
        while !bytes.is_empty() {
            wait_ready(stream, libc::POLLOUT, deadline)?;
            match stream.write(bytes) {
                Ok(0) => return Err("sync write failed"),
                Ok(n) => bytes = &bytes[n..],
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(_) => return Err("sync write failed or timed out"),
            }
        }
    }
    Ok(())
}

fn same_user(stream: &UnixStream) -> bool {
    #[cfg(target_os = "macos")]
    {
        let mut uid = 0;
        let mut gid = 0;
        // SAFETY: valid connected fd and writable uid/gid pointers.
        unsafe {
            libc::getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) == 0 && uid == libc::getuid()
        }
    }
    #[cfg(target_os = "linux")]
    {
        let mut cred: libc::ucred = unsafe { std::mem::zeroed() };
        let mut len = std::mem::size_of_val(&cred) as libc::socklen_t;
        // SAFETY: valid fd, credentials buffer and size pointer.
        unsafe {
            libc::getsockopt(
                stream.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                (&mut cred as *mut libc::ucred).cast(),
                &mut len,
            ) == 0
                && cred.uid == libc::getuid()
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = stream;
        false
    }
}

fn handle(mut stream: UnixStream, token: &str, queue: &Mutex<Queue>) {
    let deadline = Instant::now() + Duration::from_secs(2);
    let result = (|| -> Result<Reply> {
        if !same_user(&stream) {
            return Err("unauthorized sync peer");
        }
        let (request, bytes): (Request, usize) = read_frame(&mut stream, deadline)?;
        if request.version != 1 {
            return Err("unsupported sync version");
        }
        // Compare all bytes; no token content reaches diagnostics.
        if request.token.len() != token.len()
            || request
                .token
                .bytes()
                .zip(token.bytes())
                .fold(0, |n, (a, b)| n | (a ^ b))
                != 0
        {
            return Err("unauthorized sync token");
        }
        validate_entries(&request.env)?;
        let env: Vec<_> = request
            .env
            .into_iter()
            .filter(|(k, _)| !reserved(k))
            .collect();
        let mut queue = queue.lock().map_err(|_| "sync queue unavailable")?;
        if queue.entries.len() >= MAX_QUEUE || queue.bytes + bytes > MAX_QUEUED_BYTES {
            return Err("sync queue full");
        }
        let sequence = queue
            .sequence
            .checked_add(1)
            .ok_or("sync sequence exhausted")?;
        queue.sequence = sequence;
        queue.bytes += bytes;
        let accepted = env.len();
        queue.entries.push_back(Pending { sequence, env });
        Ok(Reply {
            version: 1,
            status: "queued".into(),
            sequence,
            accepted,
        })
    })();
    let reply = result.unwrap_or_else(|reason| Reply {
        version: 1,
        status: reason.into(),
        sequence: 0,
        accepted: 0,
    });
    let _ = write_frame(&mut stream, &reply, deadline);
}

impl Server {
    pub fn start(data: &Path) -> Result<Self> {
        let id = random_hex(12)?;
        let dir = socket_dir(data, &id)?;
        Self::bind(dir.join(format!("{id}.sock")))
    }

    fn bind(path: PathBuf) -> Result<Self> {
        if path.to_str().is_none() {
            return Err("sync socket path is not UTF-8");
        }
        let endpoint = Endpoint {
            path,
            token: random_hex(32)?,
        };
        endpoint.validate()?;
        let listener = UnixListener::bind(&endpoint.path).map_err(|_| "cannot bind sync socket")?;
        let setup = (|| -> Result<()> {
            fs::set_permissions(&endpoint.path, fs::Permissions::from_mode(0o600))
                .map_err(|_| "cannot protect sync socket")?;
            listener
                .set_nonblocking(true)
                .map_err(|_| "cannot configure sync socket")
        })();
        if let Err(err) = setup {
            let _ = fs::remove_file(&endpoint.path);
            return Err(err);
        }
        let queue = Arc::new(Mutex::new(Queue::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let (work_queue, work_stop, token) = (queue.clone(), stop.clone(), endpoint.token.clone());
        let thread = thread::Builder::new()
            .name("nutorch-sync".into())
            .spawn(move || {
                let mut workers: Vec<JoinHandle<()>> = Vec::new();
                while !work_stop.load(Ordering::Acquire) {
                    let mut i = 0;
                    while i < workers.len() {
                        if workers[i].is_finished() {
                            let _ = workers.swap_remove(i).join();
                        } else {
                            i += 1;
                        }
                    }
                    match listener.accept() {
                        Ok((stream, _)) if workers.len() < 8 => {
                            let (queue, token) = (work_queue.clone(), token.clone());
                            if let Ok(worker) = thread::Builder::new()
                                .name("nutorch-sync-client".into())
                                .spawn(move || handle(stream, &token, &queue))
                            {
                                workers.push(worker);
                            }
                        }
                        Ok(_) => (), // Capacity reached: close promptly, no unbounded waiting.
                        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(10))
                        }
                        Err(_) => break,
                    }
                }
                for worker in workers {
                    let _ = worker.join();
                }
            });
        match thread {
            Ok(thread) => Ok(Self {
                endpoint,
                queue,
                stop,
                thread: Mutex::new(Some(thread)),
                owner_pid: std::process::id(),
            }),
            Err(_) => {
                let _ = fs::remove_file(&endpoint.path);
                Err("cannot start sync receiver")
            }
        }
    }

    pub fn drain(&self) -> Vec<Pending> {
        if let Ok(mut queue) = self.queue.lock() {
            queue.bytes = 0;
            queue.entries.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    pub fn shutdown(&self) {
        if self.owner_pid != std::process::id() {
            return;
        }
        self.stop.store(true, Ordering::Release);
        if let Ok(mut thread) = self.thread.lock() {
            if let Some(thread) = thread.take() {
                let _ = thread.join();
                let _ = fs::remove_file(&self.endpoint.path);
            }
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.shutdown();
    }
}

static EXIT_SERVER: OnceLock<Arc<Server>> = OnceLock::new();
extern "C" fn cleanup() {
    if let Some(server) = EXIT_SERVER.get() {
        server.shutdown();
    }
}

pub fn register_exit(server: Arc<Server>) -> Result<()> {
    // SAFETY: process-lifetime, non-unwinding callback; registered once per shell.
    if unsafe { libc::atexit(cleanup) } != 0 {
        return Err("cannot register sync cleanup");
    }
    EXIT_SERVER
        .set(server)
        .map_err(|_| "sync receiver already initialized")
}

pub fn send(endpoint: &Endpoint, env: Vec<(String, String)>) -> Result<Reply> {
    endpoint.validate()?;
    validate_entries(&env)?;
    private_dir(
        endpoint.path.parent().ok_or("invalid sync directory")?,
        false,
    )?;
    let metadata = fs::symlink_metadata(&endpoint.path)
        .map_err(|_| "enclosing NuTorch sync socket is unavailable")?;
    use std::os::unix::fs::FileTypeExt;
    if !metadata.file_type().is_socket() {
        return Err("sync target is not a socket");
    }
    let request = Request {
        version: 1,
        token: endpoint.token.clone(),
        env: env.into_iter().filter(|(k, _)| !reserved(k)).collect(),
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    let socket = socket2::Socket::new(socket2::Domain::UNIX, socket2::Type::STREAM, None)
        .map_err(|_| "cannot create sync client")?;
    let address = socket2::SockAddr::unix(&endpoint.path).map_err(|_| "invalid sync address")?;
    socket
        .connect_timeout(&address, remaining(deadline)?)
        .map_err(|_| "cannot connect to enclosing NuTorch")?;
    let fd: std::os::fd::OwnedFd = socket.into();
    let mut stream = UnixStream::from(fd);
    if !same_user(&stream) {
        return Err("unauthorized sync receiver");
    }
    write_frame(&mut stream, &request, deadline)
        .map_err(|_| "sync send failed; acceptance unknown, not retried")?;
    let (reply, _): (Reply, _) = read_frame(&mut stream, deadline)
        .map_err(|_| "sync acknowledgment missing; acceptance unknown, not retried")?;
    if reply.version != 1
        || reply.status != "queued"
        || reply.sequence == 0
        || reply.accepted > MAX_ENTRIES
    {
        return Err("sync request rejected by enclosing NuTorch");
    }
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (tempfile::TempDir, Server) {
        let dir = tempfile::Builder::new()
            .prefix("ntsync-")
            .tempdir_in("/tmp")
            .unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let server = Server::start(dir.path()).unwrap();
        (dir, server)
    }
    fn request(server: &Server, env: Vec<(String, String)>) -> Request {
        Request {
            version: 1,
            token: server.endpoint.token.clone(),
            env,
        }
    }
    fn raw(server: &Server, request: &Request) -> Reply {
        let mut stream = UnixStream::connect(&server.endpoint.path).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        write_frame(&mut stream, request, deadline).unwrap();
        read_frame(&mut stream, deadline).unwrap().0
    }
    #[test]
    fn roundtrip_order_limits_and_cleanup() {
        let (_dir, server) = fixture();
        assert_eq!(
            fs::metadata(&server.endpoint.path).unwrap().mode() & 0o777,
            0o600
        );
        for i in 1..=32 {
            let reply = send(
                &server.endpoint,
                vec![
                    ("A".into(), format!("{i}")),
                    ("PWD".into(), "ignored".into()),
                ],
            )
            .unwrap();
            assert_eq!(reply.sequence, i);
            assert_eq!(reply.accepted, 1);
        }
        assert!(send(&server.endpoint, vec![]).is_err());
        let snapshots = server.drain();
        assert_eq!(snapshots.len(), 32);
        for (i, snapshot) in snapshots.iter().enumerate() {
            assert_eq!(snapshot.env, vec![("A".into(), format!("{}", i + 1))]);
        }
        assert_eq!(send(&server.endpoint, vec![]).unwrap().sequence, 33);
        server.shutdown();
        assert!(!server.endpoint.path.exists());
        assert!(send(&server.endpoint, vec![]).is_err());
    }
    #[test]
    fn rejects_bad_protocol_without_changing_queue() {
        let (_dir, server) = fixture();
        let mut bad = request(&server, vec![]);
        bad.version = 2;
        assert_ne!(raw(&server, &bad).status, "queued");
        bad.version = 1;
        bad.token = "0".repeat(64);
        assert_ne!(raw(&server, &bad).status, "queued");
        for entries in [
            vec![("A".into(), "1".into()), ("A".into(), "2".into())],
            vec![("".into(), "x".into())],
            vec![("a=b".into(), "x".into())],
            vec![("A".into(), "\0".into())],
            vec![("A".into(), "x".repeat(MAX_VALUE + 1))],
            (0..4097)
                .map(|i| (format!("V{i}"), String::new()))
                .collect(),
        ] {
            assert_ne!(raw(&server, &request(&server, entries)).status, "queued");
        }
        for bytes in [
            vec![0, 16, 0, 1],
            vec![0, 0, 0, 3, b'{', b'}'],
            vec![0, 0, 0, 1, 0xff],
        ] {
            let mut stream = UnixStream::connect(&server.endpoint.path).unwrap();
            stream.write_all(&bytes).unwrap();
            stream.shutdown(std::net::Shutdown::Write).unwrap();
            let reply: Reply = read_frame(&mut stream, Instant::now() + Duration::from_secs(3))
                .unwrap()
                .0;
            assert_ne!(reply.status, "queued");
        }
        assert!(server.drain().is_empty());
        assert!(send(&server.endpoint, vec![("GOOD".into(), "yes".into())]).is_ok());
    }
    #[test]
    fn byte_budget_deadlines_and_lost_ack() {
        let (_dir, server) = fixture();
        let large: Vec<_> = (0..15)
            .map(|i| (format!("V{i}"), "x".repeat(MAX_VALUE)))
            .collect();
        for _ in 0..8 {
            send(&server.endpoint, large.clone()).unwrap();
        }
        assert!(send(&server.endpoint, large.clone()).is_err());
        assert_eq!(server.drain().len(), 8);
        let too_large = (0..17)
            .map(|i| (format!("V{i}"), "x".repeat(MAX_VALUE)))
            .collect();
        assert!(send(&server.endpoint, too_large).is_err());
        assert!(server.drain().is_empty());
        // An accepted request survives loss of its acknowledgment, without retry.
        let mut stream = UnixStream::connect(&server.endpoint.path).unwrap();
        write_frame(
            &mut stream,
            &request(&server, vec![]),
            Instant::now() + Duration::from_secs(1),
        )
        .unwrap();
        drop(stream);
        let until = Instant::now() + Duration::from_secs(2);
        loop {
            let entries = server.drain();
            if !entries.is_empty() {
                assert_eq!(entries.len(), 1);
                break;
            }
            assert!(Instant::now() < until);
            thread::sleep(Duration::from_millis(10));
        }
        let start = Instant::now();
        let mut slow = UnixStream::connect(&server.endpoint.path).unwrap();
        slow.write_all(&[0]).unwrap();
        let mut byte = [0];
        wait_ready(&slow, libc::POLLIN, Instant::now() + Duration::from_secs(3)).unwrap();
        let _ = slow.read(&mut byte);
        assert!(start.elapsed() < Duration::from_secs(3));
        assert!(server.drain().is_empty());
        send(&server.endpoint, vec![]).unwrap();
    }
    #[test]
    fn data_security_and_independent_sessions() {
        let (dir, one) = fixture();
        let two = Server::start(dir.path()).unwrap();
        assert_ne!(one.endpoint.path, two.endpoint.path);
        send(&one.endpoint, vec![("A".into(), "one".into())]).unwrap();
        assert!(two.drain().is_empty());
        assert_eq!(one.drain().len(), 1);
        assert!(Server::bind(one.endpoint.path.clone()).is_err());
        one.shutdown();
        assert!(two.endpoint.path.exists());
        send(&two.endpoint, vec![("B".into(), "still alive".into())]).unwrap();
        let private = dir.path().join("astrohacker/nutorch");
        fs::set_permissions(&private, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(Server::start(dir.path()).is_err());
        fs::set_permissions(&private, fs::Permissions::from_mode(0o700)).unwrap();
        let link = dir.path().join("link");
        fs::create_dir(&link).unwrap();
        std::os::unix::fs::symlink(dir.path().join("astrohacker"), link.join("astrohacker"))
            .unwrap();
        assert!(Server::start(&link).is_err());
        let long = dir.path().join("x".repeat(100));
        assert!(Server::start(&long).is_err());
        assert!(!long.exists());
        assert!(Server::start(Path::new("relative")).is_err());
    }

    #[test]
    fn xdg_defaults_and_ordinary_shared_root() {
        let home = Path::new("/Users/example");
        for data in [None, Some(Path::new("")), Some(Path::new("relative"))] {
            assert_eq!(
                data_home(data, Some(home)).unwrap(),
                home.join(".local/share")
            );
            assert!(data_home(data, None).is_err());
            assert!(data_home(data, Some(Path::new("relative"))).is_err());
        }
        assert_eq!(
            data_home(Some(Path::new("/custom data")), None).unwrap(),
            Path::new("/custom data")
        );
        let (dir, server) = fixture();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(
            dir.path().join("astrohacker"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        let two = Server::start(dir.path()).unwrap();
        assert!(
            two.endpoint
                .path
                .starts_with(dir.path().join("astrohacker/nutorch"))
        );
        server.shutdown();
        assert!(two.endpoint.path.exists());
        let unicode = dir.path().join("é space");
        let three = Server::start(&unicode).unwrap();
        assert!(three.endpoint.path.starts_with(&unicode));
    }

    #[test]
    fn active_client_capacity_recovers_after_deadlines() {
        let (_dir, server) = fixture();
        let mut clients = Vec::new();
        for _ in 0..8 {
            let mut stream = UnixStream::connect(&server.endpoint.path).unwrap();
            stream.write_all(&[0]).unwrap();
            clients.push(stream);
        }
        thread::sleep(Duration::from_millis(100));
        let mut overflow = UnixStream::connect(&server.endpoint.path).unwrap();
        let start = Instant::now();
        wait_ready(&overflow, libc::POLLIN, start + Duration::from_secs(1)).unwrap();
        let mut byte = [0];
        assert_eq!(overflow.read(&mut byte).unwrap(), 0);
        assert!(start.elapsed() < Duration::from_secs(1));
        drop(clients);
        thread::sleep(Duration::from_millis(100));
        send(&server.endpoint, vec![]).unwrap();
        assert_eq!(server.drain().len(), 1);
    }
}
