---
title: Autograd ops
description: The 4 autograd operations, generated from the op table.
order: 27
section: "Reference"
---

Generated from the binaries by `scripts/gen-ops-reference.ts` — do not edit by
hand. Every op also documents itself: `torch <op> --help`.

### backward

backpropagate from a scalar loss (gradients accumulate)

```bash
torch backward <t1>
```

```nu
nutorch backward <t1>
```

### grad

snapshot of a tensor's accumulated gradient

```bash
torch grad <t1>
```

```nu
nutorch grad <t1>
```

### detach

a graph-free reference to the same data

```bash
torch detach <t1>
```

```nu
nutorch detach <t1>
```

### zero_grad

zero a tensor's accumulated gradient in place

```bash
torch zero_grad <t1>
```

```nu
nutorch zero_grad <t1>
```
