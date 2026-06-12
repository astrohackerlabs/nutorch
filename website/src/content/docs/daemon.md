---
title: The daemon
description: nutorchd owns the tensors and the GPU. It starts itself, renews its lease on every operation, and retires after an idle TTL.
order: 2
section: Core
---

`nutorchd` is the process that owns the tensor registry, the LibTorch context,
GPU memory, and autograd graphs. Clients talk to it over a Unix socket. Its
lifecycle is invisible plumbing: you never start it, and by default it cleans up
after itself.

## Auto-start and the idle TTL

Any `torch` command starts the daemon if it isn't running. It shuts itself down
after **1 hour of inactivity** — and every tensor operation renews the lease, so
the clock only runs while you're idle. Tensors live exactly as long as the
daemon: that is the **memory-horizon contract**. Export what you want to keep
(see [tensors](/docs/tensors/)); everything else returns to the GPU when the
daemon retires.

The default TTL is configurable via `NUTORCHD_TTL` (e.g. `30m`, `2h`, `none` for
no expiry).

## Inspecting and controlling it

```bash
torch daemon status      # pid, ttl, idle time, time remaining,
                         # tensor count, memory held, socket, log
torch daemon ttl 4h      # change the idle TTL on the live daemon (none = forever)
torch daemon stop        # shut down now
torch daemon restart     # fresh daemon, empty registry
torch daemon start       # start without running an op
```

`torch daemon status --json` emits the same record as JSON for scripts.

A status report looks like:

```
version: 0.1.0
pid: 32799
uptime: 240s
device: mps
ttl: 3600s
idle: 12s
remaining: 3588s
tensors: 3
memory: ~24 bytes
socket: /tmp/…/nutorchd.sock
log: /tmp/…/nutorchd.log
```

## Concurrency

Multiple shells can talk to one daemon at the same time. Connections are served
concurrently (thread per connection); operations execute strictly serialized
inside the daemon, so a parked or slow client never blocks anyone else's
commands.

## GPU-only, by design

The daemon requires MPS and refuses to start without it. There is no device
option anywhere in the API — every tensor lives on the Apple-silicon GPU.
