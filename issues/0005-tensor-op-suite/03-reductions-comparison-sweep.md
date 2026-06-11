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

# Experiment 3: Reductions + comparison sweep (~35 ops), and variable-arity results

## Description

Second and third categories on the loom. One genuine spec extension rides along,
declared here rather than smuggled: **`ResultKind::VariableHandles`**, for ops
whose _output count depends on their params_ — `max`/`min` return one tensor as
a full reduce but (values, indices) with `--dim`; `median` likewise. The
dispatch debug-assert and usage text adapt; the client already prints however
many handles arrive. (This is the issue's predicted "extend the spec, note it in
that experiment" path.)

MPS support is decided by the golden-generator oracle, as in Experiment 2;
exclusions recorded.

## Changes

1. **`ops/src/lib.rs`**:
   - `ResultKind::VariableHandles` + row constructors `reduction(...)`,
     `binary_compare(...)` (Bool result, broadcasting), `unary_predicate(...)`
     (Bool result);
   - **reductions (~18)**:
     `prod amax amin max min argmax argmin all any std
     var nansum logsumexp count_nonzero cumsum cumprod median norm`
     — `--dim`/`--keepdim` where PyTorch has them; `cumsum`/`cumprod`/
     `logsumexp` require `--dim`; `std`/`var` get `--correction` (Int, default
     1, PyTorch 2.x semantics); `norm` gets `--p` (Float, default 2) and
     optional `--dim`; `max`/`min`/`median` are `VariableHandles` (1 without
     `--dim`, 2 with); `all`/`any` return Bool tensors.
   - **comparison (~17)**: binary Bool broadcasting `gt lt ge le ne isclose`
     (`--rtol`/`--atol`); unary Bool
     `isnan isinf isfinite isposinf isneginf
     logical_not`; binary Bool
     `logical_and logical_or logical_xor`; `equal` (whole-tensor equality →
     `Value` bool); `topk` (`k` positional Int, optional `--dim`, returns
     values+indices — `Handles(2)`); `argsort` (`--dim`/`--descending`).
     **Recorded spec limitation**: presence-only Bool flags cannot override a
     true-default — PyTorch's `topk(largest=True)` default is therefore the only
     behavior reachable via a faithful `--largest` flag. Decision: omit
     `--largest` and ship a `--smallest` Bool flag mapped to `largest=false`,
     with its summary marking it a nutorch-ism (PyTorch spells it
     `largest=False`). The same pattern covers any future true-default Bool.
2. **`nutorchd/src/dispatch.rs`**: apply arms (mostly one line; `max`/`min`/
   `median` branch on `--dim`; reductions follow PyTorch dtype promotion by
   passing `None` dtype where tch allows, with goldens as the arbiter).
   - The result-count `debug_assert` gains a `VariableHandles` arm accepting
     `outputs.len() ∈ {1, 2}` (the dim-branch determines which).
   - `norm` dispatches two-armed: `f_norm_scalaropt_dim(p, dim, keepdim)` with
     `--dim`, `f_norm_scalaropt_dtype(p, dtype)` without.
   - **Non-finite predicate semantics get a Rust dispatch unit test** (the
     Experiment-2 `nan_to_num_semantics` pattern): construct
     `[NaN, inf, -inf, 1.0]` via 0-division on MPS and assert the true-path
     outputs of `isnan`, `isinf`, `isposinf`, `isneginf`, and `isfinite` —
     golden inputs must be finite, so without this an `isnan` accidentally wired
     to `isfinite` would pass every golden.
3. **`scripts/gen-golden.py`**: data-driven cases — every op ≥1 case; `--dim`
   variants for the variable-arity trio (both shapes of result); Bool-tensor
   expectations; `topk`/`argsort` cases; `equal` true/false; oracle skips
   recorded. Golden inputs finite (the Experiment-2 constraint).
4. **`nutorchd/tests/golden.rs`**: floor raised (~135+).

## Verification

1. **Hygiene**: build 0 warnings; all tests green; fmt/dprint clean on touched
   files; `v1/` untouched.
2. **Golden suite green** (~135+ cases), regeneration byte-stable.
3. **Variable arity live**: `torch max $t` → one handle;
   `torch max $t --dim
   0` → two handles, each piping to `value`; same for
   `median`.
4. **Bool families live**: `gt`, `isnan` (on a finite tensor → all false),
   `logical_not`; `equal` prints `true`/`false`. The non-finite TRUE path of the
   predicate family is guarded by the Rust unit test in Changes item 2 (cited
   here as the compensating check for the finite-goldens constraint).
5. **topk live**: `torch topk $t 2` → two handles (values, indices), exact
   against golden; `--smallest` flips selection.
6. **`torch ops` count** = table + 2 (programmatic).
7. **Oracle exclusions recorded** with Python error lines.

**Pass** = all seven. **Partial** = sweep minus recorded exclusions/follow-ups.
**Fail** = the spec extension required dispatch/client surgery beyond the
declared VariableHandles handling.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**First pass: CHANGES REQUIRED** — 1 Required: the isnan/isinf/isfinite family's
TRUE path was untested (goldens are finite by constraint; an `isnan` wired to
`isfinite` would have passed all seven checks). **Fixed:** a Rust dispatch unit
test (the Exp-2 non-finite pattern) asserts the true-path outputs of all five
predicates, cited from Verification. Two Optional, folded in: the
VariableHandles debug-assert shape is now specified (`outputs.len() ∈ {1,2}`),
and norm's two-armed dispatch is named. The reviewer verified every claimed tch
API exists at the right signature (f_std_correction, f_var_correction,
f_norm_scalaropt_{dim,dtype}, f_max_dim/f_min_dim/f_median_dim pairs, f_topk),
all ~35 ops run on MPS in the linked torch, the client is already N-handle-safe
(confirming "client unchanged"), and the `--smallest` decision is sound given
the presence-only Bool grammar. Approved with the fixes in place (re-review
folded into this record; the fixes are the reviewer's own prescriptions
verbatim).

## Result

**Result:** Pass

36 new ops landed (18 reductions, 18 comparison); the table grew 72 → 108. The
`VariableHandles` spec extension cost exactly what was declared: one enum
variant, one debug-assert arm, zero client changes.

- **Golden suite: 134/134** (was 90), first run, byte-stable regeneration
  (sha256 `02923bda…` twice); dprint-clean; floor raised to 130.
- **Hygiene**: build 0 warnings; 32 daemon unit tests (including the new
  non-finite predicate test proving the TRUE path of all five predicates on a
  constructed [NaN, inf, -inf, 1.0] MPS tensor); fmt/dprint clean; `v1/`
  untouched.
- **Live**: `max $m` → 1 handle; `max $m --dim 1` → 2 handles piping to values
  `[3.0,6.0]` and indices `[0,0]`; `median` scalar; `gt` → `[false,true,false]`;
  `isnan` finite → all false; `equal` → true/false; `topk 2` → values
  `[5.0,4.0]` + indices `[1,3]`; `--smallest` flips to `[1.0,3.0]`; `torch ops`
  = 110 = 108 + 2.
- **MPS oracle exclusions: none** — all 36 planned ops ran on MPS in the linked
  PyTorch (generation completed without a single skip).

## Conclusion

Three categories down (pointwise, reductions, comparison): 108 ops, all
golden-verified. The VariableHandles extension closes the
output-count-depends-on-params shape; the presence-only-Bool limitation got its
honest workaround (`--smallest`). Remaining sweeps: linalg + shape + indexing
(the structurally richest — split/chunk return a variable LIST of tensors,
einsum brings the Str param + variadic combination, `where` is ternary), then
creation + the deferred tensor-valued-param extension (clamp bounds, pow tensor
exponent), then close.

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only),
reviewing the pre-commit working tree. **Verdict: APPROVED — no Required,
Optional, or Nit findings.** The reviewer independently reproduced every gate:
all three design-review-mandated items confirmed real (the non-finite predicate
test runs and passes all five true paths; the VariableHandles debug-assert arm;
norm's two-armed dispatch); variable arity, topk --smallest, std --correction 0
vs 1, and dtype promotion all verified live AND against fresh independent Python
computations; goldens byte-stable at sha256 02923bda…; `all`-on-float coercion
semantics confirmed to match PyTorch; "zero MPS exclusions" verified honest
against the generator diff; plan/result commit separation clean.
