//! Explicit child-to-enclosing-shell environment synchronization.
#[cfg(test)]
mod tests;
pub mod transport;

use nu_engine::{command_prelude::*, convert_env_vars, env_to_string};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use transport::{Endpoint, Server};

pub struct Context {
    parent: transport::Result<Option<Endpoint>>,
    disabled: AtomicBool,
}

impl Context {
    pub fn capture() -> Arc<Self> {
        Arc::new(Self {
            parent: Endpoint::inherited(),
            disabled: AtomicBool::new(false),
        })
    }

    pub fn start(&self, engine: &EngineState, stack: &mut Stack) -> Option<Arc<Server>> {
        let data = stack
            .get_env_var(engine, "XDG_DATA_HOME")
            .and_then(|v| v.as_str().ok().map(std::path::PathBuf::from));
        let home = stack
            .get_env_var(engine, "HOME")
            .and_then(|v| v.as_str().ok().map(std::path::PathBuf::from));
        let result = transport::data_home(data.as_deref(), home.as_deref())
            .and_then(|base| Server::start(&base))
            .and_then(|server| {
                let server = Arc::new(server);
                transport::register_exit(server.clone())?;
                Ok(server)
            });
        match result {
            Ok(server) => {
                stack.add_env_var(
                    transport::SOCKET.into(),
                    Value::string(server.endpoint.path.to_string_lossy(), Span::unknown()),
                );
                stack.add_env_var(
                    transport::TOKEN.into(),
                    Value::string(&server.endpoint.token, Span::unknown()),
                );
                Some(server)
            }
            Err(reason) => {
                self.disabled.store(true, Ordering::Release);
                stack.remove_env_var(engine, transport::SOCKET);
                stack.remove_env_var(engine, transport::TOKEN);
                eprintln!("nutorch sync: receiver disabled: {reason}");
                None
            }
        }
    }
}

pub fn add_context(engine: &mut EngineState, context: Arc<Context>) {
    let mut working = StateWorkingSet::new(engine);
    working.add_decl(Box::new(SyncCommand(context)));
    engine
        .merge_delta(working.render())
        .expect("register nutorch sync");
}

fn exported(engine: &EngineState, stack: &Stack) -> transport::Result<Vec<(String, String)>> {
    let mut entries = Vec::new();
    for (name, value) in stack.get_env_vars(engine) {
        if transport::reserved(&name) {
            continue;
        }
        match env_to_string(&name, &value, engine, stack) {
            Ok(value) => entries.push((name, value)),
            Err(ShellError::EnvVarNotAString { .. }) => (),
            Err(_) => return Err("environment export conversion failed"),
        }
    }
    Ok(entries)
}

// Stage the whole snapshot before changing any imported binding. Conversion closures
// may have their own external effects, just as when launching an ordinary child.
fn apply(
    engine: &EngineState,
    stack: &mut Stack,
    entries: Vec<(String, String)>,
) -> transport::Result<()> {
    engine
        .signals()
        .check(&Span::unknown())
        .map_err(|_| "environment conversion interrupted; snapshot rejected")?;
    transport::validate_entries(&entries)?;
    let mut changes = Vec::new();
    let existing: std::collections::HashMap<_, _> = stack
        .get_env_vars(engine)
        .into_iter()
        .map(|(key, value)| (key.to_ascii_uppercase(), (key, value)))
        .collect();
    for (mut name, text) in entries {
        if transport::reserved(&name) {
            continue;
        }
        if let Some((spelling, current)) = existing.get(&name.to_ascii_uppercase()) {
            // Keep the receiver's export spelling (notably Unix PATH/HOME).
            name = spelling.clone();
            let current = env_to_string(&name, current, engine, stack)
                .map_err(|_| "existing environment value cannot be exported; snapshot rejected")?;
            if current == text {
                continue;
            }
        }
        changes.push((name, Value::string(text, Span::unknown())));
    }
    if changes.is_empty() {
        return Ok(());
    }
    let names: std::collections::HashSet<_> = changes
        .iter()
        .map(|(name, _)| name.to_ascii_uppercase())
        .collect();
    let mut codecs = Record::new();
    if let Some(config) = stack.get_env_var(engine, "ENV_CONVERSIONS") {
        let config = config
            .as_record()
            .map_err(|_| "invalid receiver environment codecs; snapshot rejected")?;
        for (key, codec) in config.iter() {
            let identity = key.to_ascii_uppercase();
            if !names.contains(&identity) {
                continue;
            }
            // Nushell processes aliases in record order, skipping subsequent
            // conversions only once the staged value is no longer a string.
            codecs.push(key.clone(), codec.clone());
        }
    }
    let mut staged = stack.clone();
    for (name, value) in &changes {
        staged.add_env_var(name.clone(), value.clone());
    }
    convert_env_vars(&mut staged, engine, &Value::record(codecs, Span::unknown()))
        .map_err(|_| "receiver environment conversion failed; snapshot rejected")?;
    let mut decoded = Vec::with_capacity(changes.len());
    for (name, _) in changes {
        let value = staged
            .get_env_var(engine, &name)
            .ok_or("converted environment value missing; snapshot rejected")?;
        if matches!(value, Value::Error { .. }) {
            return Err("receiver environment conversion failed; snapshot rejected");
        }
        decoded.push((name, value.clone()));
    }
    engine
        .signals()
        .check(&Span::unknown())
        .map_err(|_| "environment conversion interrupted; snapshot rejected")?;
    for (name, value) in decoded {
        stack.add_env_var(name, value);
    }
    Ok(())
}

#[derive(Clone)]
struct SyncCommand(Arc<Context>);

impl Command for SyncCommand {
    fn name(&self) -> &str {
        "nutorch sync"
    }
    fn description(&self) -> &str {
        "Queue this shell's exported environment in its enclosing NuTorch."
    }
    fn extra_description(&self) -> &str {
        "Success means queued, not applied. The enclosing shell applies updates at its next prompt. Missing variables are not deleted; shell/session control variables are excluded."
    }
    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Nothing, Type::String)])
            .category(Category::System)
    }
    fn run(
        &self,
        engine: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let result = (|| {
            if self.0.disabled.load(Ordering::Acquire) {
                return Err("sync is disabled in this shell");
            }
            let endpoint = self
                .0
                .parent
                .as_ref()
                .map_err(|e| *e)?
                .as_ref()
                .ok_or("no enclosing NuTorch sync receiver")?;
            transport::send(endpoint, exported(engine, stack)?)
        })();
        match result {
            Ok(reply) => Ok(Value::string(
                format!(
                    "Queued environment snapshot {} ({} variables)",
                    reply.sequence, reply.accepted
                ),
                call.head,
            )
            .into_pipeline_data()),
            Err(reason) => Err(ShellError::Generic(
                nu_protocol::shell_error::generic::GenericError::new(
                    "NuTorch environment sync failed",
                    reason,
                    call.head,
                ),
            )),
        }
    }
}

/// Called before Nushell argument parsing, configuration, or receiver startup.
pub fn early_cli() -> Option<i32> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_none_or(|arg| arg != "sync") {
        return None;
    }
    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        print!("{}", crate::help::sync_help());
        return Some(0);
    }
    let result = (|| {
        if args.len() != 1 {
            return Err("usage: nutorch sync [--help]");
        }
        let endpoint = Endpoint::inherited()?.ok_or("no enclosing NuTorch sync receiver")?;
        let mut entries = Vec::new();
        for (name, value) in std::env::vars_os() {
            let name = name
                .into_string()
                .map_err(|_| "environment name is not UTF-8")?;
            if transport::reserved(&name) {
                continue;
            }
            entries.push((
                name,
                value
                    .into_string()
                    .map_err(|_| "environment value is not UTF-8")?,
            ));
        }
        transport::send(&endpoint, entries)
    })();
    Some(match result {
        Ok(reply) => {
            println!(
                "Queued environment snapshot {} ({} variables)",
                reply.sequence, reply.accepted
            );
            0
        }
        Err(reason) => {
            eprintln!("nutorch sync: {reason}");
            1
        }
    })
}

pub struct Dispatcher {
    pub server: Option<Arc<Server>>,
}

impl nu_cli::ModeDispatcher for Dispatcher {
    fn execute(
        &mut self,
        mode: &str,
        command: &str,
        env: std::collections::HashMap<String, String>,
        cwd: std::path::PathBuf,
    ) -> nu_cli::ModeResult {
        crate::dispatcher::NuTorchDispatcher.execute(mode, command, env, cwd)
    }
    fn before_prompt(&mut self, engine: &mut EngineState, stack: &mut Stack) {
        if let Some(server) = &self.server {
            let pending = server.drain();
            if pending.is_empty() {
                return;
            }
            for pending in pending {
                if let Err(reason) = apply(engine, stack, pending.env) {
                    eprintln!("nutorch sync: snapshot {}: {reason}", pending.sequence);
                }
            }
            // The interrupted batch is now rejected. Consume its interrupt before
            // prompt hooks/rendering, so Ctrl-C does not poison the next command.
            // Never reset during a decoder or between snapshots in this batch.
            if engine.signals().interrupted() {
                engine.reset_signals();
            }
        }
    }
}
