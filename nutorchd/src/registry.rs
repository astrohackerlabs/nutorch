//! The tensor registry: the daemon-owned map from string handles to live
//! objects. Issue 0009 makes it kind-aware: handles are typed strings
//! (`tensor://<uuid>`, `nn://<uuid>`, `optim://<uuid>`), keyed internally
//! by bare UUID with the kind on the entry — so a wrong prefix on a real
//! object reports the true kind instead of lying with "unknown handle".
//! Issue 0006: entries carry created/touched timestamps; touching is
//! explicit (`touch`), never a side effect of reads.

use std::collections::HashMap;
use std::time::Instant;
use tch::Tensor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleKind {
    Tensor,
    Module,
    Optimizer,
}

impl HandleKind {
    pub fn prefix(self) -> &'static str {
        match self {
            HandleKind::Tensor => "tensor",
            HandleKind::Module => "nn",
            HandleKind::Optimizer => "optim",
        }
    }

    pub fn noun(self) -> &'static str {
        match self {
            HandleKind::Tensor => "tensor",
            HandleKind::Module => "module",
            HandleKind::Optimizer => "optimizer",
        }
    }

    fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "tensor" => Some(HandleKind::Tensor),
            "nn" => Some(HandleKind::Module),
            "optim" => Some(HandleKind::Optimizer),
            _ => None,
        }
    }
}

/// Why a lookup failed — mapped to error codes in dispatch
/// (Malformed → bad_argument, Unknown → unknown_handle,
/// WrongKind → wrong_kind).
#[derive(Debug)]
pub enum Lookup {
    /// No recognized `kind://` prefix (bare UUIDs are not grandfathered).
    Malformed(String),
    /// Well-formed, but nothing lives at that id.
    Unknown(String),
    /// The id exists but the prefix names the wrong kind.
    WrongKind {
        handle: String,
        actual: HandleKind,
        expected: HandleKind,
    },
}

impl Lookup {
    pub fn code(&self) -> &'static str {
        match self {
            Lookup::Malformed(_) => "bad_argument",
            Lookup::Unknown(_) => "unknown_handle",
            Lookup::WrongKind { .. } => "wrong_kind",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Lookup::Malformed(handle) => format!(
                "malformed handle: {handle} (expected tensor://<id>, nn://<id>, or optim://<id>)"
            ),
            Lookup::Unknown(handle) => format!("unknown handle: {handle}"),
            Lookup::WrongKind {
                handle,
                actual,
                expected,
            } => format!(
                "handle {handle} refers to a {}, not a {}",
                actual.noun(),
                expected.noun()
            ),
        }
    }
}

/// Parse `kind://id` into (kind, id). The id syntax is not validated
/// beyond being non-empty — the map lookup is the authority.
fn parse_handle(handle: &str) -> Result<(HandleKind, &str), Lookup> {
    let (prefix, id) = handle
        .split_once("://")
        .ok_or_else(|| Lookup::Malformed(handle.to_string()))?;
    let kind =
        HandleKind::from_prefix(prefix).ok_or_else(|| Lookup::Malformed(handle.to_string()))?;
    if id.is_empty() {
        return Err(Lookup::Malformed(handle.to_string()));
    }
    Ok((kind, id))
}

fn mint(kind: HandleKind, id: &str) -> String {
    format!("{}://{}", kind.prefix(), id)
}

/// One stored object. Module/Optimizer variants arrive with their own
/// experiments (issue 0009).
pub enum Object {
    Tensor(Tensor),
    Module(crate::nn::NnModule),
}

impl Object {
    fn kind(&self) -> HandleKind {
        match self {
            Object::Tensor(_) => HandleKind::Tensor,
            Object::Module(_) => HandleKind::Module,
        }
    }
}

pub struct Entry {
    pub object: Object,
    pub created: Instant,
    pub touched: Instant,
}

/// One row of `list()`: everything `torch tensors` shows (tensor rows).
pub struct Listing {
    pub handle: String,
    pub kind: HandleKind,
    pub shape: Vec<i64>,
    pub dtype: tch::Kind,
    pub bytes: u64,
    pub age_secs: u64,
    pub idle_secs: u64,
}

#[derive(Default)]
pub struct Registry {
    entries: HashMap<String, Entry>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_tensor(&mut self, tensor: Tensor) -> String {
        self.insert_object(Object::Tensor(tensor))
    }

    pub fn insert_module(&mut self, module: crate::nn::NnModule) -> String {
        self.insert_object(Object::Module(module))
    }

    fn insert_object(&mut self, object: Object) -> String {
        let kind = object.kind();
        let id = uuid::Uuid::new_v4().to_string();
        let now = Instant::now();
        self.entries.insert(
            id.clone(),
            Entry {
                object,
                created: now,
                touched: now,
            },
        );
        mint(kind, &id)
    }

    fn lookup(&self, handle: &str, expected: HandleKind) -> Result<&Entry, Lookup> {
        let (kind, id) = parse_handle(handle)?;
        let entry = self
            .entries
            .get(id)
            .ok_or_else(|| Lookup::Unknown(handle.to_string()))?;
        let actual = entry.object.kind();
        if kind != actual || kind != expected {
            // Report what the object IS vs what was claimed: a wrong
            // prefix is the user's claim ("nn://" says module), so the
            // message names the prefix's kind; only when the prefix is
            // truthful does the accessor's expectation apply.
            let expected = if kind != actual { kind } else { expected };
            return Err(Lookup::WrongKind {
                handle: handle.to_string(),
                actual,
                expected,
            });
        }
        Ok(entry)
    }

    pub fn get_tensor(&self, handle: &str) -> Result<&Tensor, Lookup> {
        match &self.lookup(handle, HandleKind::Tensor)?.object {
            Object::Tensor(tensor) => Ok(tensor),
            _ => unreachable!("lookup kind-checked"),
        }
    }

    pub fn get_module(&self, handle: &str) -> Result<&crate::nn::NnModule, Lookup> {
        match &self.lookup(handle, HandleKind::Module)?.object {
            Object::Module(module) => Ok(module),
            _ => unreachable!("lookup kind-checked"),
        }
    }

    /// Mark an object as used. A no-op on absent/malformed handles, so
    /// the table's touch pass stays harmless when resolution is about to
    /// error.
    pub fn touch(&mut self, handle: &str) {
        if let Ok((_, id)) = parse_handle(handle) {
            if let Some(entry) = self.entries.get_mut(id) {
                entry.touched = Instant::now();
            }
        }
    }

    /// Kind-agnostic existence check used by `free`'s validate pass.
    /// Errors carry the same Lookup shapes as typed accessors.
    pub fn check(&self, handle: &str) -> Result<(), Lookup> {
        let (kind, id) = parse_handle(handle)?;
        let entry = self
            .entries
            .get(id)
            .ok_or_else(|| Lookup::Unknown(handle.to_string()))?;
        let actual = entry.object.kind();
        if kind != actual {
            return Err(Lookup::WrongKind {
                handle: handle.to_string(),
                actual,
                expected: kind,
            });
        }
        Ok(())
    }

    /// Boolean form of `check` (test-friendly).
    pub fn check_ok(&self, handle: &str) -> bool {
        self.check(handle).is_ok()
    }

    /// Remove any kind of object by handle.
    pub fn remove(&mut self, handle: &str) -> Option<Entry> {
        let (_, id) = parse_handle(handle).ok()?;
        self.entries.remove(id)
    }

    /// Empty the registry; returns how many objects were freed.
    pub fn clear(&mut self) -> usize {
        let count = self.entries.len();
        self.entries.clear();
        count
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Approximate bytes held: Σ numel × element size over tensor entries.
    pub fn approx_bytes(&self) -> u64 {
        self.entries
            .values()
            .map(|e| match &e.object {
                Object::Tensor(t) => t.numel() as u64 * t.kind().elt_size_in_bytes() as u64,
                Object::Module(m) => m.param_bytes(),
            })
            .sum()
    }

    /// All tensor entries, oldest-created first.
    pub fn list(&self) -> Vec<Listing> {
        let now = Instant::now();
        let mut rows: Vec<(&String, &Entry)> = self.entries.iter().collect();
        rows.sort_by_key(|(_, entry)| entry.created);
        rows.into_iter()
            .filter_map(|(id, entry)| match &entry.object {
                Object::Module(_) => None,
                Object::Tensor(t) => Some(Listing {
                    handle: mint(HandleKind::Tensor, id),
                    kind: HandleKind::Tensor,
                    shape: t.size(),
                    dtype: t.kind(),
                    bytes: t.numel() as u64 * t.kind().elt_size_in_bytes() as u64,
                    age_secs: now.duration_since(entry.created).as_secs(),
                    idle_secs: now.duration_since(entry.touched).as_secs(),
                }),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn handles_are_minted_with_the_tensor_prefix() {
        let mut registry = Registry::new();
        let a = registry.insert_tensor(Tensor::from(1.0));
        assert!(a.starts_with("tensor://"), "got {a}");
        assert!(registry.get_tensor(&a).is_ok());
    }

    #[test]
    fn the_four_error_shapes() {
        let mut registry = Registry::new();
        let a = registry.insert_tensor(Tensor::from(1.0));

        // 1. Malformed: bare UUID (the clean break) and unknown prefix.
        for bad in [
            "not-a-handle",
            "9a3c2f00-0000-0000-0000-000000000000",
            "blob://x",
        ] {
            match registry.get_tensor(bad) {
                Err(Lookup::Malformed(_)) => {}
                other => panic!("{bad}: expected Malformed, got {other:?}"),
            }
        }
        // 2. Well-formed, absent id.
        match registry.get_tensor("tensor://does-not-exist") {
            Err(Lookup::Unknown(_)) => {}
            other => panic!("expected Unknown, got {other:?}"),
        }
        // 3. Wrong prefix on a REAL object.
        let id = a.strip_prefix("tensor://").unwrap();
        match registry.get_tensor(&format!("nn://{id}")) {
            Err(Lookup::WrongKind {
                actual, expected, ..
            }) => {
                assert_eq!(actual, HandleKind::Tensor);
                // The message blames the PREFIX's claim, not the accessor.
                assert_eq!(expected, HandleKind::Module);
            }
            other => panic!("expected WrongKind, got {other:?}"),
        }
        // check() reports the PREFIX's kind as expected for free's UX.
        match registry.check(&format!("nn://{id}")) {
            Err(Lookup::WrongKind { actual, .. }) => assert_eq!(actual, HandleKind::Tensor),
            other => panic!("expected WrongKind, got {other:?}"),
        }
        // 4. The happy path still happy.
        assert!(registry.get_tensor(&a).is_ok());
    }

    #[test]
    fn lookup_error_codes_and_messages() {
        let registry = Registry::new();
        let err = registry.get_tensor("bare").unwrap_err();
        assert_eq!(err.code(), "bad_argument");
        assert!(err.message().contains("expected tensor://"));
        let err = registry.get_tensor("tensor://gone").unwrap_err();
        assert_eq!(err.code(), "unknown_handle");
    }

    #[test]
    fn list_is_oldest_first_with_correct_fields() {
        let mut registry = Registry::new();
        let a = registry.insert_tensor(Tensor::from_slice(&[1.0f32, 2.0, 3.0]));
        std::thread::sleep(Duration::from_millis(15));
        let b = registry.insert_tensor(Tensor::from_slice(&[1i64, 2]));
        let rows = registry.list();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].handle, a);
        assert_eq!(rows[1].handle, b);
        assert_eq!(rows[0].shape, vec![3]);
        assert_eq!(rows[0].dtype, tch::Kind::Float);
        assert_eq!(rows[0].bytes, 12);
        assert_eq!(rows[1].dtype, tch::Kind::Int64);
        assert_eq!(rows[1].bytes, 16);
    }

    #[test]
    fn touch_resets_idle_but_get_does_not() {
        let mut registry = Registry::new();
        let a = registry.insert_tensor(Tensor::from(1.0));
        std::thread::sleep(Duration::from_millis(1100));
        assert!(registry.list()[0].idle_secs >= 1);
        let _ = registry.get_tensor(&a); // reads do not touch
        assert!(registry.list()[0].idle_secs >= 1);
        registry.touch(&a);
        assert_eq!(registry.list()[0].idle_secs, 0);
        assert!(registry.list()[0].age_secs >= 1);
        registry.touch("not-a-handle"); // no-op, no panic
    }
}
