---
title: The NuTorch shell
description: Launch NuTorch, import torch and work with native tensors in a full Nushell-based shell.
order: 7
section: Start
---

## Setup

Build and launch the unreleased shell from the Astrohacker checkout:

```nu
nu scripts/build.nu nutorch --release
./code/nutorch/rs/target/release/nutorch
```

Run `use torch` inside NuTorch to bring the native tensor commands into scope.
Ordinary shell commands work immediately without this import.
The embedded module needs no client file. Do not import `nutorch.nu` or
edit `NU_LIB_DIRS`; the client module has been removed. An existing installed
legacy executable is not this development shell.

`use torch` keeps the `torch ...` prefix. `use torch tensor` imports just
`tensor`, while `use torch *` imports unqualified names such as `add` and `sum`
with normal Nushell shadowing rules. Compatibility `nutorch ...` commands require
`use torch nutorch`. Prefer the qualified `use torch` form in scripts.

Scripts explicitly import their dependencies. To make torch available in every
interactive session, you may add `use torch` to your NuTorch `config.nu`.
It is not imported by default. The import controls command scope and preserves
the existing libtorch loading behavior; it does not reset or free native values.

Normal mode evaluates Nushell, including structured pipelines, functions, scripts,
completion, history and external commands. Shift+Tab switches to an unfinished
AI mode whose input is not evaluated; switching modes clears the current buffer.
Ctrl+C cancels native loops between commands and leaves the shell usable.

## Native structured values

```nu
use torch
let t = torch tensor [[1 2] [3 4]]
let container = {tensor: $t}
$container.tensor | torch mm $t | torch value
```

The result is `[[7.0 10.0] [15.0 22.0]]`. The record retains the same tensor
rather than serializing a handle or copying the GPU data.

## Scripts

```nu
./code/nutorch/rs/target/release/nutorch code/nutorch/examples/train-regression.nu
./code/nutorch/rs/target/release/nutorch code/nutorch/examples/train-classify.nu
```

Run scripts with the new shell so its native commands are present.
Plain Nushell does not gain tensor commands by installing a module.
Live tensors, modules and optimizers belong to their process and cannot be
passed to external programs as handles.
