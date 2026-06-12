---
title: Reduction ops
description: The 21 reduction operations, generated from the op table.
order: 23
section: "Reference"
---

Generated from the binaries by `scripts/gen-ops-reference.ts` — do not edit by
hand. Every op also documents itself: `torch <op> --help`.

### sum

sum over all elements, or along --dim

```bash
torch sum <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch sum <t1> [--dim <Int>] [--keepdim]
```

### mean

mean over all elements, or along --dim (float32, v1 fidelity)

```bash
torch mean <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch mean <t1> [--dim <Int>] [--keepdim]
```

### prod

product over all elements, or along --dim

```bash
torch prod <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch prod <t1> [--dim <Int>] [--keepdim]
```

### amax

max values over all elements, or along --dim

```bash
torch amax <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch amax <t1> [--dim <Int>] [--keepdim]
```

### amin

min values over all elements, or along --dim

```bash
torch amin <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch amin <t1> [--dim <Int>] [--keepdim]
```

### max

max of all elements; with --dim also returns indices

```bash
torch max <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch max <t1> [--dim <Int>] [--keepdim]
```

### min

min of all elements; with --dim also returns indices

```bash
torch min <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch min <t1> [--dim <Int>] [--keepdim]
```

### median

median of all elements; with --dim also returns indices

```bash
torch median <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch median <t1> [--dim <Int>] [--keepdim]
```

### argmax

index of the max, overall or along --dim

```bash
torch argmax <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch argmax <t1> [--dim <Int>] [--keepdim]
```

### argmin

index of the min, overall or along --dim

```bash
torch argmin <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch argmin <t1> [--dim <Int>] [--keepdim]
```

### all

true if all elements are true (Bool tensor)

```bash
torch all <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch all <t1> [--dim <Int>] [--keepdim]
```

### any

true if any element is true (Bool tensor)

```bash
torch any <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch any <t1> [--dim <Int>] [--keepdim]
```

### std

standard deviation (--correction, default 1)

```bash
torch std <t1> [--dim <Int>] [--keepdim] [--correction <Int>]
```

```nu
nutorch std <t1> [--dim <Int>] [--keepdim] [--correction <Int>]
```

### var

variance (--correction, default 1)

```bash
torch var <t1> [--dim <Int>] [--keepdim] [--correction <Int>]
```

```nu
nutorch var <t1> [--dim <Int>] [--keepdim] [--correction <Int>]
```

### nansum

sum treating NaN as zero

```bash
torch nansum <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch nansum <t1> [--dim <Int>] [--keepdim]
```

### logsumexp

log(sum(exp(x))) along --dim

```bash
torch logsumexp <t1> [--dim <Int>] [--keepdim]
```

```nu
nutorch logsumexp <t1> [--dim <Int>] [--keepdim]
```

### count_nonzero

count of nonzero elements, overall or along --dim

```bash
torch count_nonzero <t1> [--dim <Int>]
```

```nu
nutorch count_nonzero <t1> [--dim <Int>]
```

### cumsum

cumulative sum along --dim

```bash
torch cumsum <t1> [--dim <Int>]
```

```nu
nutorch cumsum <t1> [--dim <Int>]
```

### cumprod

cumulative product along --dim

```bash
torch cumprod <t1> [--dim <Int>]
```

```nu
nutorch cumprod <t1> [--dim <Int>]
```

### norm

p-norm (--p default 2), overall or along --dim

```bash
torch norm <t1> [--p <Float>] [--dim <Int>] [--keepdim]
```

```nu
nutorch norm <t1> [--p <Float>] [--dim <Int>] [--keepdim]
```

### diff

forward differences along --dim (default last)

```bash
torch diff <t1> [--dim <Int>]
```

```nu
nutorch diff <t1> [--dim <Int>]
```
