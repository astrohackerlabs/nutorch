---
title: Operations
description: 185 ops with PyTorch names, PyTorch argument order, and PyTorch semantics — discoverable from the shell.
order: 4
section: Core
---

nutorch's operation surface is a single declarative table shared by the daemon
and the CLI: **185 ops** spanning creation, pointwise math, linear algebra,
reductions, shape manipulation, indexing, losses, and more.

## PyTorch fidelity

Command names, argument order, defaults, and semantics match PyTorch wherever
possible. If you know `torch.add(a, b, alpha=2)`, you know:

```bash
torch add $a $b --alpha 2
```

Broadcasting follows PyTorch's rules. Non-broadcastable shapes error with both
shapes named — validation happens in Rust before any GPU call, so error messages
talk about your tensors, not C++ internals.

## Discoverability

```bash
torch ops                # every op: name, category, one-line summary
torch ops --json         # the same as JSON (name, category, summary)
torch mean --help        # usage, parameters, defaults for any op
```

Every op supports the dual input pattern — pipe the leftmost tensor in or pass
it as an argument (see [tensors](/docs/tensors/)).

## Reference

A generated per-op reference — built from the same table the binaries use, so it
cannot drift — is coming to this site in the next stage of issue 0012. Until it
lands, `torch ops` and `torch <op> --help` are the reference, and they are
always current.
