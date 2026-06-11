+++
[implementer]
agent = "claude-code"
model = "claude-fable-5"

[review.design]
agent = "claude-code"
subagent = "adversarial-reviewer"
model = "claude-opus"

[review.result]
agent = "claude-code"
subagent = "adversarial-reviewer"
model = "claude-opus"
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

## Result

**Result:** Pass

31 new ops landed (12 creation, 19 remainder); the table grew 141 → 172. The
`HandleOrScalar` extension cost exactly the three declared sites (one enum
variant + client parse arm + dispatch resolution/branches) and discharged both
recorded deferrals: `clamp` takes tensor bounds, `pow` takes a tensor exponent,
and `lerp` shipped with both weight forms from day one.

- **Golden suite: 206/206** (was 170); byte-stable (sha256 `bac61e5f…` twice);
  dprint-clean; floor 200. The harness gained the `T<i>` convention: a golden
  param value `"T1"` resolves to input tensor 1's handle and drops it from the
  operand list — how param-tensors are golden-tested.
- **Hygiene**: build 0 warnings; fmt clean; all tests green; `v1/` untouched.
- **Live**: `zeros`/`arange --start --step`/`eye`; clamp with tensor AND scalar
  bounds; pow both exponent forms; lerp both weight forms; bitwise int64; seeded
  `rand` deterministic; `scatter` exact; a typo'd scalar surfaces as honest
  `unknown_handle`; `torch ops` = 174 = 172 + 2.
- **MPS oracle exclusions: none this sweep.** `empty`/`empty_like` excluded
  permanently by design (nondeterministic contents; recorded).
- **Two tch-binding notes**: `addcmul`/`addcdiv` expose no `value` param in tch
  0.24, so the scaled form is computed manually (`a + value*(b∘c)`; goldens pin
  parity with PyTorch's fused kwarg). ATen's `searchsorted` binds `self` to the
  VALUES tensor, so the dispatch flips operands to keep the CLI faithful to
  `torch.searchsorted(sorted_seq, values)`.
- **Two silent-replace escapes caught by the goldens** during implementation:
  the clamp param retype and the clamp tensor-bounds branch both initially
  no-op'd against fmt-drifted text; the golden `cr_clamp_tensor_bounds` case
  caught each within seconds. The golden pipeline continues to pay for itself.

## Conclusion

The op surface is complete per the issue's scope: **172 table ops across six
categories** (pointwise, reduction, comparison, linalg, shape, creation), every
one golden-verified against the exact PyTorch the daemon links, plus the bespoke
tensor/value/daemon verbs. Every structural shape the issue predicted is
implemented and exercised: dual results, variable results, list results,
variadic tensors, Str/IntList/Bool/Scalar params, and tensor-or-scalar hybrid
params. Three MPS exclusions stand recorded (heaviside, take — upstream gaps;
empty — by design). The long tail (special functions, the deeper fft/linalg
namespaces) is table-row work the loom makes mechanical whenever wanted. The
issue can close.

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only),
reviewing the pre-commit working tree. **First pass: CHANGES REQUIRED** — one
Required finding, and a textbook one: `unique` via `f_unique_dim(-1)` does not
flatten, so it diverged from `torch.unique` for rank ≥ 2 while the 1-D-only
golden passed vacuously (live evidence: `[[3,1],[2,1]]` returned `[[1,3],[1,2]]`
where PyTorch returns `[1,2,3]`). **Fixed with the reviewer's prescription
verbatim** (flatten → unique dim 0) plus a rank-2 golden pinning the contract;
207/207 green, rank-2 live check returns `[1.0,2.0,3.0]`. Everything else held
under attack: HandleOrScalar end-to-end live in all forms, addcmul value=2
bitwise-identical to fresh PyTorch, the searchsorted operand flip, tensordot
axis order on a non-symmetric case, byte-stable goldens at the recorded sha, the
Fail criterion (three declared sites) honored, registry orphan-freedom, and the
172+2 count parsed from the OPS static. **Close readiness:** the reviewer
audited every promise across the issue and its experiments — AGENTS.md
amendments (exp 1), the README pointer, both deferred tensor-param extensions
(here) — and found nothing undischarged beyond the unique fix.
