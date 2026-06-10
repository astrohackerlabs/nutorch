+++
[implementer]
agent = "claude-code"
model = "claude-fable-5"
+++

# Experiment 3: The compute ops — `full`, `add`, `mm`, `mean`, and the two PoC pipelines

## Description

Complete the issue's six-op set on the Experiment-2 spine and run the issue's
two PoC pipelines end-to-end from a plain shell, on MPS, with exact expected
values. After this experiment the issue's goal statement is satisfied: create a
tensor or two, perform an operation on them on the GPU, get back the result.

Experiment 2's conclusion scoped this precisely: the four ops are dispatcher
entries plus argument plumbing, with validation ported from
`v1/cargo/src/command_*.rs` and every tch call fallible (`f_*`).

## Changes

1. **Protocol** (`nutorchd/src/protocol.rs`): four new request variants —
   - `{"op":"full","shape":[i64...],"value":<number>,"device":...,"dtype":...}`
     → handle (defaults as in `tensor`: cpu, float32);
   - `{"op":"add","a":"<handle>","b":"<handle>"}` → handle;
   - `{"op":"mm","a":"<handle>","b":"<handle>"}` → handle;
   - `{"op":"mean","handle":"<handle>"}` → handle (a 0-D tensor goes into the
     registry, so `mean` pipes into `value` like everything else).

2. **Dispatch** (`nutorchd/src/main.rs`):
   - `full`: validate shape non-empty and all dims ≥ 1 (Rust-side, per the
     carried-forward validation principle); `Tensor::f_full` with the scalar
     fill (integer JSON number → int scalar, float → double scalar, then cast to
     the requested kind/device as in `tensor`).
   - `add`: look up both handles; **Rust-side device-equality check ported from
     v1 `command_add.rs:140-148`** (a clean "device mismatch" error naming both
     devices, before any tch call — carried-forward principle 5; no
     auto-transfer); then `a.f_add(b)`.
   - `mm`: the same Rust-side device check, plus validation ported from v1
     `command_mm.rs:117-140`: both tensors must be rank-2 and inner dimensions
     must match (`[n,k]·[k,m]`), with a shape-spelling error message; then
     `a.f_mm(b)`.
   - `mean`: `t.f_mean(Kind::Float)` — v1 fidelity: v1 defaults the mean dtype
     to float32 via `get_kind_from_call` regardless of the input tensor's kind
     (`command_mean.rs:133,152`, `lib.rs:197`), which also defines the int-input
     behavior (an int tensor is reduced in float32, as in v1) and stays MPS-safe
     (no float64).
   - All four insert their result into the registry and return the new handle
     (v1 semantics: every op output is a new registry entry).

3. **Client** (`torch-cli/src/main.rs`): four new ops with the established
   dual-input plumbing —
   - `full <shape-json> <value> [--device][--dtype]` (both args positional; no
     stdin form — creation ops take no tensor input, matching `tensor`);
   - `add [a] [b]`, `mm [a] [b]`: two handles; with two positionals use them as
     (a,b); with one positional, **a comes from stdin** and the positional is b
     (pipeline form: `... | torch mm "$(...)"` reads the piped handle as the
     left operand, matching v1's pipeline-is-first-operand convention); with
     zero positionals, error;
   - `mean [handle]`: positional XOR stdin, like `value`.

4. **Unit tests** (`nutorchd/src/main.rs` `#[cfg(test)]`, in-process via
   `handle_request` + a `Registry`, no socket): full→value round trip
   (`full [2,2] 1` → `[[1.0,1.0],[1.0,1.0]]`); add exact (`[1,2,3]+[4,5,6]` →
   `[5.0,7.0,9.0]`); mm exact 2×2 ones → all 2.0; mean exact; mm rank/shape
   rejection ([2,3]·[2,3] errors mentioning shapes); add unknown-handle error;
   full with empty shape or zero/negative dim errors.

## Verification

From the repo root; the behavioral checks run against a daemon started directly
(not via cargo) on a dedicated `--socket` path, torn down (kill + socket file
removed) at the end.

1. **Hygiene**: `cargo build` 0 warnings; `cargo test` green (Experiment 1+2
   tests plus the new dispatch tests); `cargo fmt --all -- --check` clean;
   `dprint check` clean on files this experiment touches;
   `git status --porcelain v1/` empty.
2. **PoC pipeline 1 (the issue's round trip), exact**:
   ```bash
   a=$(torch tensor '[1,2,3]' --device mps)
   b=$(torch tensor '[4,5,6]' --device mps)
   torch add $a $b | torch value     # → [5.0,7.0,9.0]
   ```
3. **PoC pipeline 2 (the issue's GPU showcase), exact**:
   ```bash
   torch full '[1000,1000]' 1 --device mps \
     | torch mm "$(torch full '[1000,1000]' 1 --device mps)" \
     | torch mean | torch value      # → 1000.0
   ```
   Exactness rationale: each mm element sums 1000 f32 ones (< 2^24, exact); the
   mean's reduction over identical values composes sums that stay exactly
   representable in practice (v1's README demo showed exactly this on MPS at
   20000²). If MPS reduction nonetheless drifts, that is recorded and assessed
   honestly (Partial), not papered over with a tolerance.
4. **Cross-device mismatch errors cleanly**:
   `torch add $cpu_handle
   $mps_handle` → exit 1, a one-line error **naming
   both devices** (the ported v1 Rust-side check), daemon survives (liveness
   probe).
5. **mm shape validation**: `[2,3]`-ones mm `[2,3]`-ones → exit 1, error names
   the offending shapes, daemon survives (liveness probe).
6. **CPU equivalence**: pipeline 1 without `--device mps` gives the same
   `[5.0,7.0,9.0]` (device-independence of results).
7. **Informal timing note** (not a pass criterion): time the `mm` of
   `[4000,4000]` ones on cpu vs mps via `time`, recorded in the Result for the
   README-demo lineage.

**Pass** = checks 1–6. **Partial** = compute works but an MPS-specific numeric
or placement issue forces deviation from exactness (recorded with evidence).
**Fail** = either PoC pipeline produces a wrong value for any reason other than
the recorded MPS-numeric deviation above.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**First pass: CHANGES REQUIRED** — 1 Required, 3 Optional:

- [Required] The `add` design dropped v1's Rust-side device-mismatch check
  (`command_add.rs:140-148` errors naming both devices before any tch call)
  while mislabeling the drop as "explicit v1-style behavior" — the rationale was
  factually inverted. **Fixed:** the v1 device check is ported for both `add`
  and `mm`, and verification check 4 now requires the error to name both
  devices.
- [Optional] The mean rationale falsely claimed v1 operated in the tensor's own
  kind; v1 defaults the mean dtype to float32 via `get_kind_from_call`.
  **Fixed:** restated accurately with citations.
- [Optional] Int-input `mean` behavior unspecified. **Fixed:** int tensors
  reduce in float32, as in v1.
- [Optional] Partial/Fail overlap for an MPS-numeric drift. **Fixed:** Fail now
  excludes the recorded MPS-numeric carve-out.

**Re-review (fresh context): APPROVED** — all four findings confirmed resolved
against the v1 source (citations verified exact); the reviewer also confirmed
the added mm device check is honestly attributed (v1's mm has no device check;
the design credits it to `add`'s) and in-scope per Experiment 2's conclusion. No
new findings.
