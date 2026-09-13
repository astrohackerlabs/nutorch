---
title: Getting started
description: Build the unreleased NuTorch shell and run native GPU tensor pipelines.
order: 1
section: Start
---

NuTorch is a full Nushell-based shell with tensors built into its main process.
These pages describe the **unreleased native shell**. Homebrew currently provides
the earlier tensor-tool release; it does not install this new shell.

## Build the development shell

From the Astrohacker checkout with its pinned dependencies:

```nu
nu scripts/build.nu nutorch --release
./code/nutorch/rs/target/release/nutorch
```

Use the local build explicitly. Do not import an old tensor client into it.
See [source setup](/docs/install-from-source/) and
[NuTorch shell setup](/docs/nushell/#setup).

You are now at the NuTorch prompt. Ordinary shell commands work immediately.
Type `use torch` to enable tensor commands. The examples below run inside NuTorch;
each includes the import so it also works in a fresh session.

## Your first tensor

```nu
use torch
torch tensor [1 2 3] | torch value
```

The result is ordinary data `[1.0 2.0 3.0]`. Without `torch value`, display
shows bounded tensor metadata rather than downloading the elements.

## Compose operations

```nu
use torch
torch tensor [1 2 3] | torch mul (torch tensor [2 2 2]) | torch value
```

This computes `[2.0 4.0 6.0]`. Tensor operands are native values, not handle strings.
Assignment, lists, records and function arguments share ownership.

## Differentiate

```nu
use torch
let x = torch tensor [1 2 3] --requires-grad
$x | torch mul $x | torch sum | torch backward
$x | torch grad | torch value
```

The gradient is `[2.0 4.0 6.0]`. Continue with [tensors](/docs/tensors/),
[autograd](/docs/autograd/) or [neural networks](/docs/neural-networks/).
