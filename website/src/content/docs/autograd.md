---
title: Autograd
description: Gradients flow through any pipeline. backward accumulates, grad snapshots, zero_grad resets, detach stops tracking.
order: 5
section: Deep learning
---

LibTorch records the computation graph automatically once a tensor requires
gradients — and that works across separate `torch` invocations, because the
tensors (and the graph) live in the daemon, not in your shell.

```bash
w=$(torch randn '[3]' --requires_grad)
loss=$(torch mul $w $w | torch sum)
torch backward $loss            # gradients ACCUMULATE across calls
torch grad $w | torch value     # a snapshot: later backwards won't change it
torch zero_grad $w              # reset; grad now reads as zeros
d=$(torch detach $w)            # graph-free reference (stops tracking)
```

## Rules of the road

These are PyTorch's rules, surfaced with shell-friendly errors:

- **`backward` needs a scalar loss** on a tensor that requires gradients —
  reduce first (`sum`, `mean`, or a loss op).
- **`grad` before any backward is an error**, not a silent zero.
- **Rebuild the graph before each backward.** Re-run the pipeline that produces
  your loss each iteration; a second backward through the SAME graph errors,
  exactly as in PyTorch.
- **Gradients accumulate** across backward calls until you `zero_grad` — again,
  exactly as in PyTorch.

## Graph lifetime and handles

Freeing an intermediate's handle is safe: the graph holds its tensors internally
until the graph itself dies. But **keep your leaf handles** — they are the only
key to their gradients. `torch tensors` counts only registry handles; graph-held
storage is invisible to it.

## Losses are ordinary ops

```bash
torch mse_loss $pred $target | torch backward
```

`cross_entropy`, `l1_loss`, `binary_cross_entropy_with_logits`, and friends are
all in the table — see `torch ops` under `loss`. For full training loops with
optimizers, see [neural networks](/docs/neural-networks/).
