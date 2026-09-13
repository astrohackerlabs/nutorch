---
title: Install from source
description: Build the development shell from the Astrohacker checkout while standalone distribution remains unqualified.
order: 8
section: Start
---

The native shell is unreleased. Its standalone public-source export, Homebrew
installation and TermSurf default-shell integration remain unqualified.
Current Homebrew packages are the earlier tensor-tool product.

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
Source-export and publication helpers refuse before external effects until the
shell/fork distribution is qualified. A local build is not installation proof.
