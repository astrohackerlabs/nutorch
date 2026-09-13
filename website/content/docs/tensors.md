---
title: Tensors
description: Create native MPS tensors, share their ownership and explicitly convert their data.
order: 3
section: Core
---

These commands run inside the unreleased native NuTorch shell.

## Creation and dtypes

```nu
use torch
let x = torch tensor [[1 2] [3 4]]
let integers = torch tensor [9007199254740993 -7] --dtype int64
let flags = torch tensor [true false]
torch shape $x
```

Numeric input defaults to float32; all-bool input infers bool. Mixed bool/numeric
input needs an explicit dtype. Input lists must be nonempty and rectangular.
Supported dtype names and aliases retain the baseline rules. MPS does not support
float64 execution, so that request errors. There is no CPU or device-selection mode.

## Native identity

Continue in the same session after **Creation and dtypes**, using `$x`.

```nu
use torch
let alias = {nested: [$x]}.nested.0
$alias | torch add $x | torch value
```

The record shares ownership of the tensor. Display shows shape, dtype, device and
gradient metadata without downloading all elements. Generic serialization produces
a display summary; raw custom-value serialization and plugin transport are rejected.

## Explicit data conversion

Continue with `$integers` from **Creation and dtypes**.

```nu
use torch
let exported = ($integers | torch value --meta)
let restored = ($exported | torch tensor)
$restored | torch tolist
```

`torch value` and `torch tolist` return native scalars/lists.
`--meta` returns a `{dtype, shape, data}` record for validated reconstruction.
A conflicting `--dtype` or a shape that disagrees with the data is an error.
Int64 values keep their integer precision; booleans remain booleans. Native
non-finite floats round-trip explicitly. Legacy data tokens `NaN`, `Infinity`
and `-Infinity` are also accepted by tensor creation.

## Tensor lifetime

Dropping a variable releases its reference. Other aliases, module parameters,
optimizers or autograd graphs may still retain storage. There is no registry-wide
free command; no alias can invalidate another live alias. Allocator caches may
retain GPU memory after the last reference is released.
