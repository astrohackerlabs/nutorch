---
title: Operations
description: Use all 185 native tensor operations through shared command metadata and structured pipelines.
order: 4
section: Core
---

`torch ops` lists all 185 native tensor operations. The [reference](/docs/reference/creation/)
is generated from the local release shell's metadata. Commands cover creation
and RNG, pointwise math, comparisons, reductions, shape/indexing, linear algebra,
losses and autograd.

## Input forms

```nu
use torch
let a = torch tensor [1 2 3]
let b = torch tensor [4 5 6]
torch add $a $b --alpha 2
$a | torch add $b --alpha 2
[$a $b] | torch stack
torch cat [$a $b]
```

A pipeline supplies leading tensor operands. Positional operation parameters
follow the tensor operands; flags retain their baseline defaults. Lists of
native tensors replace lists of handle strings. Extra or ambiguous operands
fail. After `use torch`, unknown `torch` subcommands produce native errors.
Before import, normal Nushell external-command resolution applies.

Continue with `$a` and `$b` from **Input forms**. Tensor-or-scalar parameters
accept native tensor values or numbers:

```nu
use torch
$a | torch pow 2
$a | torch pow $b
$a | torch clamp --min (torch tensor [2 2 2])
```

## Output forms

Continue in the same session with `$a` from **Input forms**.

Most operations return one tensor. Fixed multi-output operations return an ordered
list; for example, sort returns values then indices. Variable-cardinality
operations such as split, chunk and max return lists even for one result.

```nu
use torch
let sorted = ($a | torch sort)
$sorted.0 | torch value
$sorted.1 | torch value
$a | torch split 2 | each { torch value $in }
```

Predicates such as allclose return ordinary booleans. Backward, zero_grad and
manual_seed return nothing. The old computational `nutorch ...` aliases remain,
through `use torch nutorch`, but `torch` is the canonical command family.
