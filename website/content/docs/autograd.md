---
title: Autograd
description: Differentiate native tensor computations while preserving shared leaves and independent gradient snapshots.
order: 5
section: Deep learning
---

Gradient tracking is set after tensor construction and transfer to MPS, so
created tracked tensors are leaves. Only floating-point tensors can track gradients.

```nu
use torch
let x = torch tensor [1 2 3] --requires-grad
$x | torch mul $x | torch sum | torch backward
let snapshot = ($x | torch grad)
$x | torch mul $x | torch sum | torch backward
print ($x | torch grad | torch value)
torch zero_grad $x
$snapshot | torch value
```

The second backward accumulates `[4.0 8.0 12.0]`. The first snapshot remains
`[2.0 4.0 6.0]` after both accumulation and zeroing.
`zero_grad` leaves a defined zero gradient on the live tensor.

`backward` requires a tracked scalar loss. An untracked tensor, nonscalar
loss, or gradient read before backward gives a useful error. Dropping an
intermediate shell wrapper does not break a surviving autograd graph.

`torch detach` returns an untracked value sharing the same storage.
It does not disable tracking on the original tensor. Ordinary assignment
shares the tensor; it does not detach or copy it.

Use [native optimizers](/docs/neural-networks/) for parameter updates.
Ctrl+C interrupts loops between commands; it does not preempt a GPU kernel.
