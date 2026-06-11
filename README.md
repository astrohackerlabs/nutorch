# Nutorch

**GPU-accelerated PyTorch tensor operations from any shell.**

Nutorch v2 is **nutorchd**: a standalone daemon that owns a tensor database
(backed by [tch-rs](https://github.com/LaurentMazare/tch-rs) / LibTorch, the C++
engine behind [PyTorch](https://pytorch.org/)) and serves thin CLI clients over
a Unix socket. Tensors are referenced by string identifiers, so handles flow
through ordinary pipelines — in bash, zsh, fish, or any other shell.
[Nushell](https://www.nushell.sh/) remains the premium client, with structured
data and native serialization.

**GPU-only, by design.** Every tensor lives on the GPU — that is the point of
the library. There is no device option. **Requires an Apple-silicon Mac** (the
GPU is MPS); the daemon refuses to start without it. Mac-only for now.

```bash
# Working today (PoC):
a=$(torch tensor '[1,2,3]')
b=$(torch tensor '[4,5,6]')
torch add $a $b | torch value
# → [5.0,7.0,9.0]   (computed on the GPU)
```

## The daemon

You never start the daemon — any `torch` command starts it automatically if it
isn't running. It shuts itself down after **1 hour of inactivity** (every tensor
operation renews the lease), so your tensors last long enough to be useful
without holding GPU memory you've stopped using. Tensors live exactly as long as
the daemon — that's the memory-horizon contract.

```bash
torch daemon status      # pid, ttl, idle time, time remaining,
                         # tensor count, memory held, socket, log
torch daemon ttl 4h      # change the idle TTL on the live daemon (none = forever)
torch daemon stop        # shut down now
torch daemon restart     # fresh daemon, empty registry
```

The default TTL is configurable via `NUTORCHD_TTL` (e.g. `30m`, `2h`, `none`).

Run `torch ops` to list every available operation, and `torch <op> --help` for
any one of them.

## Autograd

Gradients flow through any pipeline — libtorch records the graph automatically
once a tensor requires them:

```bash
w=$(torch randn '[3]' --requires_grad)
loss=$(torch mul $w $w | torch sum)
torch backward $loss            # gradients ACCUMULATE across calls
torch grad $w | torch value     # a snapshot: later backwards won't change it
torch zero_grad $w              # reset; grad now reads as zeros
d=$(torch detach $w)            # graph-free reference (stops tracking)
```

Rules of the road: `backward` needs a scalar loss (reduce first) on a tensor
that requires gradients; `grad` before any backward is an error; rebuilding the
graph (re-running the pipeline) before each backward is required — a second
backward through the SAME graph errors, as in PyTorch. Graph lifetime: freeing
an intermediate's handle is safe (the graph holds the tensor internally until
the graph itself dies), but keep your LEAF handles — they are the only key to
their gradients. `torch tensors` counts only registry handles; graph-held
storage is invisible to it.

## Saving tensors and reclaiming memory

Handles are typed strings — `tensor://<id>` today, with `nn://` (modules) and
`optim://` (optimizers) arriving in issue 0009 — so a handle in a script or a
log always says what it refers to, and using the wrong kind is a named error
rather than a mystery.

Tensors live exactly as long as the daemon (default idle TTL: 1 hour).
Persistence is shell redirection — export the tensors you care about, never the
intermediates:

```bash
torch value --meta $w > w.json          # export (dtype travels inside)
w=$(torch tensor "$(cat w.json)")       # re-import, dtype preserved
```

Reclaim memory selectively or wholesale:

```bash
torch tensors            # list: handle, shape, dtype, bytes, age, idle
torch free $t1 $t2       # free specific tensors (or pipe handles in)
torch free --all         # empty the registry
torch daemon restart     # the coarse valve: export, restart, re-import
```

Note: JSON has no NaN/Infinity, so `torch value` writes the string tokens
`"NaN"`, `"Infinity"`, and `"-Infinity"` for non-finite values, and
`torch tensor` reads them back — round-trips are lossless.

## Status

**Proof of concept working** (issue 0002): daemon, thin client, six ops
(`tensor`, `full`, `add`, `mm`, `mean`, `value`), exact GPU results from plain
bash. The architecture is being worked out in the open — see the issue tracker
at [issues/README.md](issues/README.md) and the agent contract / vision in
[AGENTS.md](AGENTS.md).

## v1: the proof of concept

Nutorch began as a Nushell plugin — 40 PyTorch operations, GPU acceleration
(CPU/CUDA/MPS), autograd, and neural-network training, all from the Nushell
command line. It proved the core idea that carries v2: tensors stay in a
Rust-owned registry, and the shell passes string handles through pipelines.

v1 is archived, frozen, and fully working in [`v1/`](v1/):

- [v1/README.md](v1/README.md) — user documentation, installation, demos
  (including screenshots in `v1/raw-images/`)
- [v1/AGENTS.md](v1/AGENTS.md) — the v1 architecture record
- [v1/TODO.md](v1/TODO.md) — v1 implementation status and quality tracking

The original pre-archive layout is available at the git tag `v1-final`.

## Copyright

Copyright (C) 2025-2026 Identellica LLC
