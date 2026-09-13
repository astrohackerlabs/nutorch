---
title: Install from source
description: Build NuTorch from the Astrohacker checkout for local development.
order: 8
section: Start
---

For normal use, [install the released native shell with Homebrew](/docs/getting-started/#install-with-homebrew).
It includes LibTorch. The instructions below are an optional development path
for working on NuTorch in the Astrohacker source checkout.

## Development checkout

The Astrohacker checkout contains the isolated `code/nutorch/rs` workspace
and pinned Nushell/Reedline product forks. Build on an Apple-silicon Mac with
the pinned LibTorch dependency available:

```nu
nu scripts/lib/nutorch/bootstrap.nu
nu scripts/build.nu nutorch --release
./code/nutorch/rs/target/release/nutorch
```

Bootstrap is separate dependency preparation. Routine builds preserve caches.
This builds the local preview; it does not install or change your default shell.
At the new NuTorch prompt, run `use torch` before using tensor commands.

## Verification

```nu
do {
  cd code/nutorch/rs
  cargo fmt --all --check
  cargo test --locked -p nutorch
  cargo test --locked --workspace
}
```

The workspace builds one product executable, `nutorch`. Its `torch` command
family is available through `use torch`. There is no external tensor CLI, daemon
or filesystem client module.
A local development build does not replace or upgrade the Homebrew installation.
