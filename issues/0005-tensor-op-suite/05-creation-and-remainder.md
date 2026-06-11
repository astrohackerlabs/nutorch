+++
[implementer]
agent = "claude-code"
model = "claude-fable-5"
+++

# Experiment 5: Creation ops, tensor-or-scalar params, and the remainder batch (~33 ops)

## Description

The closing sweep, in three strands:

1. **Creation (~12)**:
   `zeros ones eye arange linspace rand randint
   zeros_like ones_like full_like rand_like randn_like`.
   Random creation follows the Experiment-1 convention (seeded CPU generator →
   MPS transfer, golden-comparable). `empty`/`empty_like` are **excluded
   permanently**: nondeterministic contents cannot be golden-verified and have
   no shell-workflow value (recorded).
2. **The deferred spec extension — `ParamKind::HandleOrScalar`** (the last one
   the issue owes): a param that accepts a number (scalar) or a handle string
   (tensor). Client side: if the value parses as a number it travels as a
   number, else as a string assumed to be a handle. Daemon side: strings resolve
   through the registry (`unknown_handle` otherwise). Applied to **`clamp`**
   (`--min`/`--max` gain tensor bounds — `f_clamp_tensor`) and **`pow`**
   (`exponent` gains tensor form — PyTorch's `pow.Tensor_Tensor`), discharging
   both recorded deferrals.
3. **Remainder batch (~19, oracle-gated)**: `lerp` (weight is HandleOrScalar
   too) `addcmul addcdiv` (`--value`) `cross` (`--dim`) `kron` `tensordot`
   (`--dims` Int) `take_along_dim` (`--dim`) `searchsorted` `bucketize` `msort`
   `diff` (`--dim`) `scatter` (non-inplace: input+index+src, required `--dim`)
   `bitwise_and bitwise_or
   bitwise_xor bitwise_not bitwise_left_shift bitwise_right_shift`
   (int tensors) `unique` (default flags: sorted values only, `Handles(1)`).

**CLI overload note (`arange`)**: PyTorch overloads
`arange(end) / arange(start, end) / arange(start, end, step)`; fixed positional
counts can't express that, so the CLI form is
`torch arange <end> [--start S] [--step P]` — a documented reshaping of the same
semantics, defaults start=0 step=1.

After this sweep the issue closes. The conclusion will state coverage honestly:
the six-category core surface (~175 ops) is complete and every structural shape
is implemented; the long tail of esoteric ops (fft/linalg namespaces beyond the
shipped set, special functions, etc.) is recorded as table-row work the loom
makes mechanical whenever wanted.

## Changes

1. **`ops/src/lib.rs`**: `ParamKind::HandleOrScalar`; rows for all of the above;
   `clamp`/`pow` params retyped.
2. **`torch-cli`**: `HandleOrScalar` parsing (number → JSON number, else
   string).
3. **`nutorchd/src/dispatch.rs`**: param validation accepts number-or-string for
   `HandleOrScalar`; a resolver helper turns string params into registry
   tensors; arms for the new ops; `clamp`/`pow` arms branch scalar-vs-tensor.
4. **`scripts/gen-golden.py`**: cases for every survivor; seeded
   `rand`/`randint`/`*_like` mirror the CPU→MPS construction; clamp tensor
   bounds and pow tensor exponent cases; oracle exclusions recorded.
5. **Floor raised** (~205+).

## Verification

1. **Hygiene** (standard) + byte-stable regeneration.
2. **Golden suite green** (~205+).
3. **Live**: `zeros '[2,2]'`; `arange 5 --start 1 --step 2`;
   `clamp $t --min $bounds` (tensor bound); `pow $t $e` (tensor exponent);
   `lerp $a $b 0.5` and `lerp $a $b $w`; `bitwise_and` on int64 tensors; seeded
   `rand` determinism; `scatter` exact vs golden.
4. **`torch ops` count** = table + 2.
5. **Oracle exclusions recorded.**

**Pass** = all five. **Partial** = sweep minus recorded exclusions. **Fail** =
the HandleOrScalar extension required changes beyond the three declared sites.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**Verdict: APPROVED — no Required, Optional, or Nit findings (first pass).** The
reviewer attacked the crux directly — whether HandleOrScalar's param-handle
resolution would force restructuring beyond the declared sites — and proved it
sound: `registry.get()` is an `&self` borrow that coexists with the operand
`Vec<&Tensor>` borrows, the only `&mut` use happens after `apply` returns, and
the resolution lives in `execute_table` (an already-declared site). It verified
every named tch binding exists and runs on MPS (including `f_clamp_tensor`,
`f_pow(&Tensor)`, `f_scatter` non-inplace, the bitwise family), confirmed the
searchsorted/bucketize argument-order trap is correctly flagged, `unique`'s
default really is one sorted values tensor, `randint` is int64 with the stated
overload, and rand_like-by-shape-on-CPU gives golden parity. A UUID can never
parse as a number, so the HandleOrScalar client rule has no ambiguity; a typo'd
scalar becoming `unknown_handle` is an honest error.
