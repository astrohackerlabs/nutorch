---
title: Installing from source
description: Clone, bootstrap, install — relocatable binaries with libtorch resolved by a baked relative rpath.
order: 8
section: Install
---

The Homebrew tap (see [getting started](/docs/getting-started/)) is the
recommended install. Building from source is two scripts:

```bash
git clone https://github.com/nutorch/nutorch
cd nutorch
scripts/bootstrap.sh     # venv + torch 2.11.0 + release build (idempotent)
scripts/install.sh       # → ~/.nutorch (or pass a prefix); add ~/.nutorch/bin to PATH
torch --version
```

## What the scripts do

- **`bootstrap.sh`** creates a repo-local Python venv just to obtain the PyTorch
  2.11.0 wheel (the strict pairing for tch 0.24.0), symlinks `.libtorch` at the
  staged package, and runs the release build. It detects an existing valid venv
  and skips the download — safe to re-run.
- **`install.sh`** copies the binaries (`torch`, `nutorchd`, and a `nutorch`
  symlink — the two CLI names are the same tool), the four libtorch dylibs they
  need (`libtorch`, `libtorch_cpu`, `libc10`, `libomp`), and `nutorch.nu` into a
  prefix (default `~/.nutorch`).

The installed binaries are relocatable: libtorch's dylibs resolve through a
baked relative rpath — no environment variables, no checkout needed at runtime,
and the install keeps working if you delete the source tree.

## Verifying

```bash
torch --version          # nutorch 0.1.0 (<git sha>)
torch daemon status      # device: mps
```

`--version` works on GPU-less machines (it short-circuits before the MPS gate);
everything else requires Apple silicon — the daemon refuses to start without
MPS.

## Requirements

- Apple-silicon Mac (MPS)
- Rust toolchain (for the build)
- Python 3 (only for the bootstrap's wheel download)
