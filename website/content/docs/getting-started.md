---
title: Getting started
description: Install NuTorch with Homebrew and run native GPU tensor pipelines.
order: 1
section: Start
---

NuTorch is a full Nushell-based shell with tensors built into its main process.
The Homebrew package includes the native shell and LibTorch for Apple silicon
on macOS Tahoe 26.x.

## Install with Homebrew

With [Homebrew](https://brew.sh) installed, run:

```nu
brew trust astrohackerlabs/astrohacker
brew tap astrohackerlabs/astrohacker
brew install nutorch
```

Then launch the installed shell:

```nu
nutorch
```

See [NuTorch shell setup](/docs/nushell/#setup) for import options.
[Building from source](/docs/install-from-source/) is an optional development path.

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
