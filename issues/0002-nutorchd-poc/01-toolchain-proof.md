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

# Experiment 1: Toolchain proof — tch 0.24.0 + libtorch 2.11.0 builds and sees MPS

## Description

Issue 1's conclusion mandates this comes first: prove the chosen tch-rs/libtorch
pairing **compiles on this machine and exposes MPS** before any daemon design
depends on it. v1's stack (tch 0.20.0 against the Homebrew Python torch headers)
no longer builds under Xcode 26.4's clang, so this is a genuinely open question,
not a formality.

Versions re-verified at design time (2026-06-10) via the crates.io API and the
tch-rs README: **tch = 0.24.0** (max stable, = torch-sys 0.24.0), which requires
**libtorch v2.11.0** exactly.

The experiment stands up the v2 workspace skeleton (the smallest structure that
can host the proof) and a smoke test that performs the issue's exact-value
computation — an all-ones matmul — on the MPS device, in-process. No daemon, no
socket, no client: just "the stack works here."

## Changes

1. **Workspace root** `Cargo.toml` (new, repo root): a Cargo workspace with
   `members = ["nutorchd"]` and `resolver = "2"`. (The client crate joins in a
   later experiment. `v1/cargo` is its own independent crate and is NOT a member
   — `v1/` stays untouched.)

2. **`nutorchd/` crate** (new):
   - `nutorchd/Cargo.toml`: package `nutorchd`, edition 2021,
     `tch = { version = "0.24.0", features = ["download-libtorch"] }`. The
     `download-libtorch` feature makes torch-sys fetch the **exactly matching**
     libtorch (v2.11.0, macOS arm64) at build time, sidestepping the
     Homebrew-Python-torch header mismatch that killed the v1 build.
   - `nutorchd/src/main.rs`: a diagnostic stub for now — prints the tch crate
     version, whether MPS is available (`tch::utils::has_mps()`), and exits 0.
     The daemon replaces this in a later experiment.
   - `nutorchd/tests/mps_smoke.rs`: the proof, as an integration test:
     1. assert `tch::utils::has_mps()` is true;
     2. create `Tensor::ones([4, 4])` on `Device::Mps`, `matmul` it with itself,
        copy back to CPU, assert **every element == 4.0 exactly** and
        `mean == 4.0` exactly (the issue's no-tolerance verification idea, in
        miniature);
     3. assert the same on `Device::Cpu` (so a failure can be attributed to MPS
        specifically vs. the stack generally).

3. **`.gitignore`**: add `target/` (the v2 workspace build dir; v1 had its own
   nested .gitignore).

4. **Environment hazard, documented and neutralized**: the user's shell exports
   `LIBTORCH=/opt/homebrew/lib/python3.11/site-packages/torch` (a v1
   instruction). torch-sys prefers `LIBTORCH` over `download-libtorch` (verified
   in torch-sys build.rs: `prepare_libtorch_dir` checks `LIBTORCH` first and
   falls to the download only when it is unset; on macOS there is no
   system-location fallback), which would silently rebuild against the broken
   Homebrew headers. All build/test commands in this experiment therefore run
   with `env -u LIBTORCH -u LD_LIBRARY_PATH -u DYLD_LIBRARY_PATH`, and the
   restriction is recorded in the workspace root `Cargo.toml` as a comment.
   `LIBTORCH_USE_PYTORCH` (the other torch-sys route to a Python torch install)
   is confirmed unset in the live environment — a checked-clear precondition.
   Runtime loading is safe without `DYLD_LIBRARY_PATH`: torch-sys bakes the
   libtorch lib dir into the binary rpath (`-Wl,-rpath=...`). (A durable fix —
   e.g. `[env]` in `.cargo/config.toml` — is deferred until the download
   location is observed.)

5. **Fallback ladder** (applied in order only if the primary path fails, each
   step recorded in the Result):
   1. `download-libtorch` feature (primary);
   2. official libtorch v2.11.0 macOS arm64 zip from pytorch.org, with
      `LIBTORCH` pointed at it;
   3. either of the above plus `CXXFLAGS=-Wno-error=invalid-specialization`
      (targeted at the clang error that killed v1 — the flag name is taken from
      the `[-Winvalid-specialization]` group in the v1 build log and must be
      adjusted to whatever diagnostic clang actually emits if it differs);
   4. if none build: the experiment **Fails**, and the next experiment must
      reconsider versions (e.g. pin an older Xcode toolchain) before the PoC
      proceeds.

## Verification

All commands from the repo root with
`env -u LIBTORCH -u LD_LIBRARY_PATH -u DYLD_LIBRARY_PATH`:

1. **Build**: `cargo build` exits 0 **with no warnings from the workspace's own
   crates** (the AGENTS.md build-clean gate; dependency build-script noise is
   reported but not gating), and the downloaded libtorch version is recorded
   from the build output/cache path.
2. **Smoke test**: `cargo test -p nutorchd --test mps_smoke` exits 0, with the
   MPS assertion, the exact-value matmul assertions (MPS and CPU), all passing.
3. **Diagnostic stub**: `cargo run -p nutorchd` prints the version/MPS report
   and exits 0.
4. **Formatting**: `cargo fmt --all -- --check` clean; `dprint check` clean on
   the files this experiment creates/edits (`issues/0002-nutorchd-poc/*.md`,
   `.gitignore` is not a dprint type; Cargo.toml files are TOML → in scope).
5. **v1 untouched**: `git status --porcelain v1/` is empty.
6. **Versions recorded**: the Result states the exact tch, torch-sys, and
   libtorch versions that built, and which rung of the fallback ladder was
   needed.

**Pass** = builds and the smoke test passes on MPS via rung 1 or 2 (rung 3's
CXXFLAGS workaround also counts as Pass but must be prominently recorded as a
constraint for all future v2 builds).

**Partial** = builds and CPU assertions pass, but MPS is unavailable or MPS
results are wrong — the stack works but the GPU goal needs investigation.

**Fail** = no rung of the ladder produces a successful build.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**Verdict: APPROVED** — no Required findings. The reviewer independently
verified the load-bearing claims: the tch-rs README confirms the libtorch
v2.11.0 pairing; torch-sys's build.rs confirms `LIBTORCH` takes precedence over
`download-libtorch` (and that macOS has no system-location fallback, so
`env -u LIBTORCH` reliably routes to the download); torch-sys bakes the libtorch
lib dir into the binary rpath, so dropping `DYLD_LIBRARY_PATH` does not strand
the test at runtime; the tch API names exist on current tch-rs
(`utils::has_mps`, `Device::Mps`); and the exact-equality float assertions are
sound (integer-valued f32 sums far below 2^24). Two Optional findings and one
Nit, all folded in: (1) the build gate now requires no warnings from workspace
crates per AGENTS.md; (2) `LIBTORCH_USE_PYTORCH` is named as a confirmed-unset
precondition; (3) rung 3's CXXFLAGS flag name is marked as needing confirmation
against the actual clang diagnostic.

## Result

**Result:** Pass

**The headline: libtorch v2.11.0's headers compile cleanly under Xcode 26.4's
clang.** The `std::is_arithmetic` specialization error that killed the v1 build
(libtorch ~2.7-era headers) is gone in 2.11 — no CXXFLAGS workaround (rung 3)
was needed. All three smoke tests pass:

```
test cpu_ones_matmul_is_exact ... ok
test mps_is_available ... ok
test mps_ones_matmul_is_exact ... ok
```

and the diagnostic stub reports `MPS available: true`.

**Versions that built:** tch 0.24.0, torch-sys 0.24.0, libtorch v2.11.0 (the
official PyPI `torch==2.11.0` macOS arm64 wheel, Python 3.14 venv).
`cargo build` exits 0 with zero warnings from workspace crates;
`cargo fmt --all -- --check` clean; `dprint check` clean on all created/edited
TOML and markdown; `git status --porcelain v1/` empty.

**Fallback ladder, as actually traversed (a deviation from the design, recorded
honestly):**

- **Rung 1 (`download-libtorch`) — failed**, but not for the anticipated header
  reason: on Apple Silicon, torch-sys's download path fetches the _PyPI torch
  wheel_ (no libtorch zip exists for macOS arm64), and torch-sys 0.24.0 panicked
  with "Failed to find arm64 macosx wheel from pypi".
- **Rung 2 as designed (official libtorch zip) — impossible**: the design's
  assumption was wrong; pytorch.org publishes no macOS arm64 libtorch archive.
  The error message itself prescribes the correct route (tch-rs issue #629):
  pip-install torch and point `LIBTORCH` at it.
- **Adapted rung 2′ — succeeded**: a repo-local venv (`.venv-torch/`,
  gitignored) with `pip install torch==2.11.0` — a _fresh, exactly-paired_
  torch, not the stale Homebrew install that broke v1 — plus a stable
  `.libtorch` symlink to its `site-packages/torch`.

**Durable environment pin (the design's deferred fix, now implemented):**
`.cargo/config.toml` sets `LIBTORCH`, `DYLD_LIBRARY_PATH`, and `LD_LIBRARY_PATH`
to the `.libtorch` path with `relative = true` and `force = true`.
Force-override matters twice over: (a) the user's shell still exports the v1-era
Homebrew `LIBTORCH`, which torch-sys would prefer at build time; (b) at
_runtime_, macOS dyld searches `DYLD_LIBRARY_PATH` ahead of the binary's baked
rpath, so a stale Homebrew path could shadow the pinned dylibs even after a
correct build. With the config pin, plain `cargo build` / `cargo test` work in
any shell with no `env -u` incantation (verified: the gates above ran without
it). The `download-libtorch` feature was dropped from `nutorchd/Cargo.toml` — it
cannot work on this platform and `LIBTORCH` takes precedence anyway.

**One-time setup recorded** in the workspace `Cargo.toml` header comment and
`.cargo/config.toml`: create `.venv-torch`, pip-install `torch==2.11.0`, symlink
`.libtorch`. (The symlink embeds the venv's Python minor version — `python3.14`
today; recreate the symlink if the venv is rebuilt with a different Python.)

## Conclusion

The issue-1 mandate is discharged: the latest tch-rs/libtorch pairing builds and
computes correctly on MPS on this machine, with exact integer-valued results on
both devices. The v2 workspace skeleton exists (root workspace + `nutorchd`
crate) and the libtorch acquisition story is settled and pinned: **PyPI wheel in
a repo-local venv, `.libtorch` symlink, `.cargo/config.toml` force-pin** — not
Homebrew, not `download-libtorch`.

What the next experiment should be: the daemon spine — `nutorchd` listens on a
Unix socket, owns the `HashMap<String, Tensor>` registry, and a first client
round trip proves a tensor created by one connection is usable by a later one
(`tensor` → handle → `value`). The remaining ops (`full`, `add`, `mm`, `mean`)
and the two PoC pipelines follow once the spine stands.

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only),
reviewing the pre-commit working tree. **Verdict: APPROVED — no Required,
Optional, or Nit findings.** The reviewer independently reran every gate with
plain commands — notably **with the stale v1-era `LIBTORCH` still exported in
the live shell** — and all passed, empirically validating the
`.cargo/config.toml` force-override reasoning at both build time and runtime. It
confirmed `.libtorch` resolves to the venv and `version.py` reports 2.11.0;
verified the rung-1 panic message against the build log; judged both design
deviations (dropping `download-libtorch`, implementing the deferred config pin)
honestly recorded and the adapted-rung-2 **Pass** classification honest
("materially the rung-2 strategy, strictly better than rung 3"); and verified
the plan commit `fecffe4` contains design only, with the result commit correctly
not yet made.
