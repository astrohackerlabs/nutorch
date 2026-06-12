---
title: Linear algebra ops
description: The 17 linalg operations, generated from the op table.
order: 24
section: "Reference"
---

Generated from the binaries by `scripts/gen-ops-reference.ts` — do not edit by
hand. Every op also documents itself: `torch <op> --help`.

### mm

matrix multiply of two 2-D tensors

```bash
torch mm <t1> <t2>
```

```nu
nutorch mm <t1> <t2>
```

### matmul

general matrix product (batched, PyTorch broadcasting)

```bash
torch matmul <t1> <t2>
```

```nu
nutorch matmul <t1> <t2>
```

### bmm

batched matrix multiply of two 3-D tensors

```bash
torch bmm <t1> <t2>
```

```nu
nutorch bmm <t1> <t2>
```

### dot

dot product of two 1-D tensors

```bash
torch dot <t1> <t2>
```

```nu
nutorch dot <t1> <t2>
```

### outer

outer product of two 1-D tensors

```bash
torch outer <t1> <t2>
```

```nu
nutorch outer <t1> <t2>
```

### einsum

Einstein summation over --equation

```bash
torch einsum <t1>... (at least 1) [--equation <Str>]
```

```nu
nutorch einsum <t1>... (at least 1) [--equation <Str>]
```

### tril

lower triangle (--diagonal offset)

```bash
torch tril <t1> [--diagonal <Int>]
```

```nu
nutorch tril <t1> [--diagonal <Int>]
```

### triu

upper triangle (--diagonal offset)

```bash
torch triu <t1> [--diagonal <Int>]
```

```nu
nutorch triu <t1> [--diagonal <Int>]
```

### diag

diagonal of a matrix, or diagonal matrix from a vector

```bash
torch diag <t1> [--diagonal <Int>]
```

```nu
nutorch diag <t1> [--diagonal <Int>]
```

### trace

sum of the main diagonal of a 2-D tensor

```bash
torch trace <t1>
```

```nu
nutorch trace <t1>
```

### det

determinant of a square matrix

```bash
torch det <t1>
```

```nu
nutorch det <t1>
```

### inverse

inverse of a square matrix

```bash
torch inverse <t1>
```

```nu
nutorch inverse <t1>
```

### svd

singular value decomposition (U, S, V)

```bash
torch svd <t1>
```

```nu
nutorch svd <t1>
```

### solve

solve AX = B for X

```bash
torch solve <t1> <t2>
```

```nu
nutorch solve <t1> <t2>
```

### cross

vector cross product along --dim

```bash
torch cross <t1> <t2> [--dim <Int>]
```

```nu
nutorch cross <t1> <t2> [--dim <Int>]
```

### kron

Kronecker product

```bash
torch kron <t1> <t2>
```

```nu
nutorch kron <t1> <t2>
```

### tensordot

tensor contraction over the last/first --dims dims (default 2)

```bash
torch tensordot <t1> <t2> [--dims <Int>]
```

```nu
nutorch tensordot <t1> <t2> [--dims <Int>]
```
