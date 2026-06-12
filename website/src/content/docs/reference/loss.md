---
title: Loss ops
description: The 9 loss operations, generated from the op table.
order: 26
section: "Reference"
---

Generated from the binaries by `scripts/gen-ops-reference.ts` — do not edit by
hand. Every op also documents itself: `torch <op> --help`.

### mse_loss

mean squared error (--reduction mean|sum|none)

```bash
torch mse_loss <t1> <t2> [--reduction <Str>]
```

```nu
nutorch mse_loss <t1> <t2> [--reduction <Str>]
```

### l1_loss

mean absolute error (--reduction)

```bash
torch l1_loss <t1> <t2> [--reduction <Str>]
```

```nu
nutorch l1_loss <t1> <t2> [--reduction <Str>]
```

### smooth_l1_loss

smooth L1 loss (--beta, default 1.0)

```bash
torch smooth_l1_loss <t1> <t2> [--reduction <Str>] [--beta <Float>]
```

```nu
nutorch smooth_l1_loss <t1> <t2> [--reduction <Str>] [--beta <Float>]
```

### huber_loss

Huber loss (--delta, default 1.0)

```bash
torch huber_loss <t1> <t2> [--reduction <Str>] [--delta <Float>]
```

```nu
nutorch huber_loss <t1> <t2> [--reduction <Str>] [--delta <Float>]
```

### cross_entropy

cross entropy over logits vs int64 class indices

```bash
torch cross_entropy <t1> <t2> [--reduction <Str>]
```

```nu
nutorch cross_entropy <t1> <t2> [--reduction <Str>]
```

### nll_loss

negative log likelihood (log-prob inputs, int64 targets)

```bash
torch nll_loss <t1> <t2> [--reduction <Str>]
```

```nu
nutorch nll_loss <t1> <t2> [--reduction <Str>]
```

### binary_cross_entropy

BCE over probabilities in [0,1]

```bash
torch binary_cross_entropy <t1> <t2> [--reduction <Str>]
```

```nu
nutorch binary_cross_entropy <t1> <t2> [--reduction <Str>]
```

### binary_cross_entropy_with_logits

BCE over logits (the stable form)

```bash
torch binary_cross_entropy_with_logits <t1> <t2> [--reduction <Str>]
```

```nu
nutorch binary_cross_entropy_with_logits <t1> <t2> [--reduction <Str>]
```

### kl_div

KL divergence (--log_target if target is log-space)

```bash
torch kl_div <t1> <t2> [--reduction <Str>] [--log_target]
```

```nu
nutorch kl_div <t1> <t2> [--reduction <Str>] [--log_target]
```
