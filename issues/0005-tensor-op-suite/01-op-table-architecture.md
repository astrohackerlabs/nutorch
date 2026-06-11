+++
[implementer]
agent = "claude-code"
model = "claude-fable-5"
+++

# Experiment 1: The op table — architecture, grammar, errors, golden tests, and 15 representative ops

## Description

Build the loom this issue weaves on. One experiment carries: the declarative op
table (a shared crate both binaries read), the generic wire op, the stdin-prefix
argument grammar with both principle amendments, the error contract with
machine-readable codes, the golden-test pipeline against the real PyTorch in
`.venv-torch`, discoverability (`torch ops`, `torch <op> --help`), and 15 ops
chosen so that **every structural shape these six categories commonly need
appears at least once** (with the rarer shapes explicitly recorded as deferred
spec extensions, below):

| Shape                        | Representative                         |
| ---------------------------- | -------------------------------------- |
| binary, broadcasting         | `add` (migrated), `sub` (new)          |
| binary, validated (no bcast) | `mm` (migrated, keeps rank/dim checks) |
| unary                        | `sin` (new)                            |
| reduction, optional `--dim`  | `sum` (new), `mean` (migrated)         |
| variadic + `--dim`           | `cat` (new)                            |
| multi-return                 | `sort` (new: values + indices)         |
| scalar parameter             | `pow` (new: tensor ^ scalar)           |
| creation with params         | `full` (migrated), `randn` (new)       |
| comparison → Bool tensor     | `eq` (new)                             |
| scalar-value return          | `allclose` (new: returns a JSON bool)  |
| optional scalar params       | `clamp` (new: `--min`/`--max`)         |
| stateful utility             | `manual_seed` (new)                    |

Subsequent experiments are category sweeps that mostly add table rows, apply
mappings, and golden coverage. **Deferred spec extensions, recorded so the "rows
only" promise is honest** (each is designed in the sweep that first needs it, as
a small spec change, not a rewrite):

- **tensor-valued optional params** (`clamp(min=<tensor>)` — this experiment
  ships scalar bounds only) — pointwise sweep;
- any shape not representable as slots+params discovered during sweeps gets the
  same treatment: extend the spec, note it in that experiment.

**Table-level invariant**: variadic-tensor ops take ALL non-tensor parameters as
flags, never trailing positionals — with unbounded tensor slots there is no way
to tell where tensors end and a positional scalar begins. (PyTorch agrees: its
variadic ops take kwargs.)

## Changes

1. **New workspace member: `ops`** (crate `nutorch-ops`, **no tch dependency** —
   both the daemon and the thin client depend on it):
   - `OpSpec { name, category, tensors: Arity::Exactly(n) | Arity::AtLeast(n),
     params: &[ParamSpec], results: ResultKind::Handles(n) | Value | None,
     summary: &str }`;
   - `ParamSpec { name, kind: Int | Float | Scalar | IntList | Bool | Str,
     positional: bool, required: bool }`
     (`Str` exists from day one — `einsum`'s equation is in scope and the spec
     must not need surgery for it) (flags are `--name value`; positional params
     follow the tensor slots, in spec order);
   - the const `OPS` table with the 15 ops, plus lookup by name and iteration by
     category.

2. **nutorchd becomes lib + thin bin** (`src/lib.rs` exposing the existing
   modules; `main.rs` keeps only the socket/lifecycle wiring). Needed so the
   golden-test harness (an integration test) can drive dispatch in-process.

3. **Generic wire op** (`protocol.rs`):
   `{"op":"<table-op>","tensors":["h1",…],"params":{…}}` →
   `{"ok":true,"handles":["h1",…]}` (multi-return; single-return ops respond
   with one handle in the list — the client prints handles one per line, so
   multi-return composes in pipelines). `tensor`, `value`, and the
   daemon/lifecycle ops stay bespoke. `add`/`mm`/`mean`/`full` migrate to the
   table (wire-format change — the protocol is throwaway by decree). Errors gain
   a `code` field: `unknown_op`, `unknown_handle`, `shape_mismatch`,
   `bad_dtype`, `bad_argument`, `bad_request`, `torch_error` (the residual for
   unmapped tch failures).

4. **Daemon dispatch** (table-driven): validate tensor count and params against
   the spec, resolve handles, then a per-op `apply` match — typically one line
   each, e.g. `"sub" => binary(|a, b| a.f_sub(b))`. All calls fallible.
   **Broadcasting**: elementwise binary ops get a generic Rust-side
   broadcastability pre-check (right-aligned shape walk) so the error is
   `add: shapes [2,3] and [4] are not broadcastable` (`code: shape_mismatch`)
   rather than a trimmed tch internals line; tch's native broadcasting then does
   the work on the happy path. `mm` keeps its ported v1 validation. Comparison
   results serialize via a new Bool path in `tensor_to_json` (JSON true/false).
   `randn` requires float kinds (`bad_dtype` otherwise); `manual_seed` calls
   `tch::manual_seed`.

5. **Client grammar** (`torch-cli`):
   - tensor slots fill **stdin-prefix, positional-suffix**: with k positionals
     for arity n, read (n−k) lines from stdin into the first slots; **if k = n,
     stdin is never read**; if slots are missing and stdin is a TTY
     (`IsTerminal`), a usage error rather than a hang;
   - **variadic ops**: tensors = (all stdin lines, if stdin is not a TTY) +
     positionals;
   - positional params follow tensor slots (`torch pow $t 2`,
     `torch full '[2,2]' 1`); flags per spec (`--dim`, `--descending`,
     `--keepdim`, `--dtype` where the spec allows);
   - `torch ops` lists the table by category; `torch <op> --help` prints
     generated usage (slots, params, summary); errors print the daemon's
     one-liner (code available in the wire response for scripts).

6. **Principle amendments** (root `AGENTS.md`, recorded retirements in the
   issue-0003 style):
   - principle 2's "XOR enforcement" clause → the stdin-prefix grammar
     (positionals win; stdin unread when nothing is missing; v1's conflict error
     documented as retired and why);
   - principle 4's "no automatic broadcasting" reading → PyTorch broadcasting is
     the pledged semantics (the retirement note explains that fidelity outranks
     v1's caution).

7. **Golden tests**:
   - `scripts/gen-golden.py`, run with `.venv-torch/bin/python` **on the MPS
     device** — the same libtorch, same device, same kernels the daemon uses, so
     float results (including transcendentals like `sin`) are bitwise
     comparable; emits `nutorchd/tests/golden.json` (committed: deterministic,
     reviewable) with per-case: op, input tensors (as nested JSON), params,
     expected outputs **or expected error**. The generator **constructs every
     input with the dtype the daemon would assign** (float32 default for numeric
     lists — Python's own int64 inference must never leak in) and pins
     `device="mps"`; anything else produces false mismatches;
   - `nutorchd/tests/golden.rs` replays every case through the in-process
     dispatch and asserts exact agreement (values via the same JSON
     serialization; errors by code);
   - **recorded risk**: seeded-RNG parity between `tch::manual_seed` and Python
     `torch.manual_seed` on MPS is unverified; if the generators disagree,
     `randn`'s golden case falls back to daemon-side determinism (same seed
     twice → identical tensors) plus shape/dtype checks, and the gap is recorded
     in the Result — not papered over.

8. **Docs**: README gains a one-line pointer to `torch ops`; AGENTS.md Directory
   Structure gains the `ops/` crate.

## Verification

From the repo root; behavioral checks on a dedicated `--socket`; teardown kills
the daemon and removes socket/log. `T=./target/debug/torch`.

1. **Hygiene**: `cargo build` 0 warnings; `cargo test` green (existing 32 + the
   golden harness + new unit tests); `cargo fmt --all -- --check` clean;
   `dprint check` clean on touched files; `git status --porcelain v1/` empty.
2. **Golden suite green**: every committed golden case passes — at minimum one
   value case per representative op, a `mean`-dtype case (int input → float32
   result, the load-bearing v1 fidelity default), and one error case each for
   non-broadcastable `add`, bad-rank `mm`, and float64 `randn`.
3. **Grammar, live**:
   - `$T sub $a $b`, `$T tensor '[5,7]' | $T sub $b`, and
     `printf "$a\n$b\n" | $T sub` all yield the same exact result;
   - `printf "$a\n$b\n$c\n" | $T cat --dim 0` (pure stdin variadic) and
     `$T cat $a $b $c --dim 0` agree;
   - `printf garbage | $T sub $a $b` succeeds — proof stdin is not read when
     slots are full;
   - `$T sub $a` with a TTY stdin errors with usage, not a hang (manual
     observation recorded).
4. **Multi-return composes**: `$T sort $t --dim 0` prints two handles on two
   lines; each pipes into `$T value`; values and indices match the golden.
5. **Bool tensors**: `$T eq $a $b | $T value` prints JSON booleans.
6. **Errors with codes**: non-broadcastable `add` → one line naming op and
   shapes; wire response carries `"code":"shape_mismatch"` (verified via `nc`);
   unknown op → `unknown_op`.
7. **Determinism**: `manual_seed 42` + `randn '[2,2]'` twice (re-seeding
   between) → identical values; Python-parity per the recorded-risk plan.
8. **Discoverability**: `torch ops` output is **complete against the table**
   (programmatic count match, not eyeball); `torch sort --help` shows
   slots/params/flags.
9. **Migrated ops unchanged behaviorally**: the issue-0002 PoC pipelines still
   produce `[5.0,7.0,9.0]` and `1000.0` exactly.

**Pass** = all nine (with the randn golden fallback allowed if the RNG-parity
risk fires, recorded). **Partial** = the table works but a representative shape
needs recorded follow-up. **Fail** = the grammar misroutes operands, goldens
disagree with the daemon, or a migrated op regresses.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**First pass: CHANGES REQUIRED** — 1 Required, 3 Optional:

- [Required] The "every structural shape appears at least once" claim was false:
  no `Str` param kind (einsum is in scope), no encoding for tensor-valued params
  (clamp's tensor bounds are in scope), no op exercising a scalar-value return,
  and variadic+positional-scalar undefined — undermining the "subsequent
  experiments only add rows" promise that justifies the experiment's size.
  **Fixed:** `Str` added to ParamSpec from day one; `allclose` (Value return)
  and `clamp` (optional scalar params) added as ops 14–15; tensor-valued params
  explicitly recorded as a deferred spec extension assigned to the pointwise
  sweep; the variadic flags-only invariant stated; the claim softened honestly.
- [Optional] Golden-generator dtype pinning under-specified (Python's int64
  inference vs the daemon's float32 default → false failures). **Fixed:** the
  generator constructs inputs with the daemon's dtype convention and pins
  device=mps.
- [Optional] Variadic + trailing positional scalars undefined. **Fixed:**
  table-level invariant (flags only).
- [Optional] Verification gaps. **Fixed:** mean-dtype golden case and a
  programmatic torch-ops-vs-table completeness check added.

**Second pass: CHANGES REQUIRED** — the fixes left three stale "13" counts
(title, Changes item 1, README link) against the now-15-op table. **Fixed.**

**Third pass (fresh context): APPROVED** — all locations consistent at 15; the
table independently recounted; remaining references verified count-agnostic. The
reviewer also confirmed (first pass): the venv is torch 2.11.0; MPS reseed
determinism holds (validating the recorded randn fallback); the broadcast walk
is correct PyTorch semantics; the current Bool-tensor path errors cleanly today,
so the new Bool serialization is necessary and regression-free; and the lib+bin
restructure is mechanical.
