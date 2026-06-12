---
title: Tensors and handles
description: Handles are typed strings. Stdin fills the leftmost tensor slot. Export with value --meta, reclaim memory with free.
order: 3
section: Core
---

Tensor data never crosses the process boundary. Clients hold **handles** —
opaque typed strings like `tensor://6c0e3f…` — and pass them between operations.
Modules and optimizers get their own schemes (`nn://`, `optim://`), so a handle
in a script or a log always says what it refers to, and using the wrong kind is
a named error rather than a mystery.

## The dual input pattern

Every operation accepts tensors both ways:

```bash
torch add $a $b                   # argument form
cat handles.txt | torch add $b    # pipeline form
```

The rule is the **stdin prefix grammar**: stdin fills the leftmost missing
tensor slots, one handle per line, and is never read when nothing is missing.
That makes `while read` loops and nested pipelines behave exactly as a shell
user expects.

## Creating tensors

```bash
torch tensor '[[1,2],[3,4]]'        # from JSON (nested lists)
torch full '[2,3]' 7                # shape, fill value
torch randn '[3,3]'                 # seeded RNG ops: also rand, randint, …
torch arange 10 --start 0 --step 2  # [0.0,2.0,4.0,6.0,8.0]
```

Run `torch ops` and look at the `creation` category for the full set; every op
documents itself with `torch <op> --help`.

## Export and import — persistence is redirection

Tensors live exactly as long as the daemon. To keep one, export it; to restore
it, import it:

```bash
torch value --meta $w > w.json          # export (dtype travels inside)
w=$(torch tensor "$(cat w.json)")       # re-import, dtype preserved
```

JSON has no NaN/Infinity, so `torch value` writes the string tokens `"NaN"`,
`"Infinity"`, and `"-Infinity"` for non-finite values, and `torch tensor` reads
them back — round-trips are lossless.

## Census and reclaiming memory

```bash
torch tensors            # list: handle, shape, dtype, bytes, age, idle
torch tensors --json     # the same, as JSON
torch free $t1 $t2       # free specific tensors (or pipe handles in)
torch free --all         # empty the registry
torch daemon restart     # the coarse valve: export, restart, re-import
```

## Errors that name things

Shapes, dims, and dtypes are validated in Rust before any GPU call, so a
non-broadcastable `add` tells you both shapes instead of crashing in C++.
Broadcasting itself follows PyTorch's rules — the semantics PyTorch users
already know are the semantics you get.
