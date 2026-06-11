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

# Experiment 1: The relocatable substrate — versioned binaries that run anywhere

## Description

Everything the formula will need, proven locally first: relocatable binaries, a
real version, and scripts that replace the Cargo.toml folklore. Ground truth
that shapes the design (verified):

- **`torch` links only libSystem** — the thin client is ALREADY fully
  relocatable; only `nutorchd` carries the libtorch problem.
- The repo lives at `github.com/nutorch/nutorch`, so the eventual tap is
  `nutorch/homebrew-nutorch` (`brew tap nutorch/nutorch`).
- `.libtorch/lib` is 237MB across 7 dylibs; at least `libtorch_python.dylib`
  (the Python binding) is plausibly dead weight — measured, not assumed (design
  question 2, answered here).

**Decisions, made here:**

1. **One install layout serves the keg and manual installs** (design question
   1):

   ```
   <prefix>/bin/torch
   <prefix>/bin/nutorchd
   <prefix>/libexec/libtorch/lib/*.dylib
   <prefix>/share/nutorch/nutorch.nu
   ```

   `nutorchd` gains a THIRD baked rpath — `@loader_path/../libexec/libtorch/lib`
   — alongside the two dev rpaths in `.cargo/config.toml`. The loader tries
   rpaths in order, so dev builds keep resolving via the checkout's `.libtorch`
   and installed binaries resolve keg-relative. No `install_name_tool`
   re-rpathing, no per-destination surgery: the SAME binary works in both worlds
   (the wheel's dylibs use `@rpath/` install names — verified premise,
   re-checked in implementation; ONE exception recorded by review:
   `libomp.dylib`'s own LC_ID is an absolute `/opt/llvm-openmp/...` path, but
   `libtorch_cpu` references it AS `@rpath/libomp.dylib`, so rpath search still
   resolves it — and `libomp` MUST be in the keg subset despite nutorchd not
   linking it directly).
2. **Version story**: the workspace bumps to **0.1.0** (one version,
   workspace-inherited so the crates cannot drift). `torch --version` and
   `nutorchd --version` print `nutorch <semver> (<short git sha>)` — the sha via
   a tiny `build.rs` env stamp, falling back to `unknown` outside a git checkout
   (source tarballs). `torch daemon status` gains a `version` field (and the
   text rendering a line), so a running daemon is identifiable — version skew
   between client and daemon becomes diagnosable. **`nutorchd
   --version` is
   handled BEFORE `require_mps()`** (review finding): version reporting is pure
   diagnostics and must work on GPU-less machines (brew test, CI). The build.rs
   sha stamp declares `cargo:rerun-if-changed` on `.git/HEAD` and the active ref
   so it cannot go stale across commits.
3. **`scripts/bootstrap.sh`** replaces the Cargo.toml-comment folklore:
   idempotently creates `.venv-torch` (python3 -m venv + torch==2.11.0) and the
   `.libtorch` symlink, then `cargo build --release`. The Cargo.toml header
   comment now points at the script.
4. **`scripts/install.sh [prefix]`** (default `~/.nutorch`): copies the release
   binaries to `bin/`, the REQUIRED dylib subset to `libexec/libtorch/lib/`, and
   `nutorch.nu` to `share/nutorch/`; prints the PATH hint. The dylib subset is
   decided by MEASUREMENT: `otool -L` on `nutorchd` for direct links, then the
   transitive closure over the dylibs themselves. The reviewer pre-measured it
   (the Result must independently confirm): KEEP `libtorch.dylib` +
   `libtorch_cpu.dylib` + `libc10.dylib` + `libomp.dylib` (~208MB; libomp only
   via libtorch_cpu); DROP `libtorch_python.dylib` (28M) + `libshm.dylib` +
   `libtorch_global_deps.dylib`. The same subset list feeds the formula in
   Experiment 2 (single source: the install script, which the formula will
   invoke or mirror).
5. **The relocatability PROOF is adversarial**: after installing to a temp
   prefix, `.libtorch` is RENAMED AWAY (and `.venv-torch` with it), so the dev
   rpaths cannot resolve; the installed `nutorchd` must start, serve a real op
   through MPS, and the full PoC pipeline must run via the installed `torch`.
   Then the rename is reverted and the dev suite re-run (both worlds keep
   working).

## Changes

1. **`Cargo.toml` (workspace)**: `[workspace.package] version = "0.1.0"`; member
   crates inherit (`version.workspace = true`). Header comment points at
   `scripts/bootstrap.sh`.
2. **`.cargo/config.toml`**: the third rpath in rustflags.
3. **`nutorchd/build.rs` + `torch-cli/build.rs`** (tiny, shared logic):
   `NUTORCH_GIT_SHA` env stamp.
4. **`torch-cli/src/main.rs`**: `--version`/`version` handling (early, like
   `ops`); **`nutorchd/src/main.rs`**: `--version`; the banner and `status` gain
   the version (dispatch's status arm + the client's `print_status` line +
   `--json` passthrough for free).
5. **`scripts/bootstrap.sh`**, **`scripts/install.sh`** (new, executable).
6. **`README.md`**: a NEW install-from-source section (none exists today) around
   the two scripts. `Cargo.lock` regenerates with the 0.1.0 bump (expected
   delta, noted so it surprises nobody).
7. **Daemon-touching note**: unlike issues 0007/0010 this issue has no
   daemon-untouched mandate; the `status` version field is a deliberate, small
   wire ADDITION (old clients ignore unknown fields — the parser is
   field-selective).

## Verification

1. **Hygiene**: build 0 warnings (debug AND release); fmt/dprint clean; full
   suite green (unit + 255 goldens + smoke); `v1/` untouched.
2. **Version**: `torch --version` and `nutorchd --version` print
   `nutorch 0.1.0 (<sha>)`; `torch daemon status` shows the version line;
   `--json` carries it.
3. **The adversarial relocation proof** (the experiment's headline): bootstrap →
   install to a mktemp prefix → rename `.libtorch` and `.venv-torch` away → from
   an unrelated cwd, the installed `torch` auto-spawns the installed `nutorchd`
   (NUTORCHD_BIN unset — sibling discovery), runs `tensor → add → value`
   correctly on MPS, and `otool -l` on the installed binary shows the
   keg-relative rpath → rename back; dev suite still green.
4. **Subset honesty**: the dropped dylibs are named in the Result with the
   measured reference closure; the installed tree's size is reported (expect
   meaningfully under 237MB).
5. **Bootstrap idempotency, with the mechanism specified** (design-review
   finding — the first draft was self-contradictory about downloads):
   `bootstrap.sh` DETECTS an existing usable `.venv-torch` and skips the pip
   download (that is what makes "idempotent" real); running it twice in this
   checkout is clean and fast. The fresh-clone simulation symlinks THIS
   checkout's `.venv-torch` into the temp clone first, so bootstrap exercises
   its wiring (symlink creation, build) without any torch fetch, then passes
   `cargo test -p nutorch-ops` (the crate with no tch dependency).

**Pass** = all five. **Fail** = one layout could not serve both worlds
(re-rpathing surgery required after all), or the subset broke MPS at runtime.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**First pass: CHANGES REQUIRED** — 1 Required: the fresh-clone idempotency
criterion was self-contradictory ("bootstraps … to avoid double-downloading
torch" — but bootstrap downloads torch by definition); the mechanism is now
specified (bootstrap detects an existing venv and skips the download; the
simulation symlinks this checkout's venv in). Optionals folded:
`nutorchd --version` runs BEFORE `require_mps()` (diagnostics must work on
GPU-less machines); the build.rs sha stamp declares rerun-if-changed on
`.git/HEAD` (staleness hazard); the `libomp` exception recorded (absolute LC_ID,
resolved via libtorch_cpu's `@rpath/` reference — a non-obvious KEEPER a naive
direct-link copy would omit, breaking MPS). Nits folded (README section is new,
not rewritten; the Cargo.lock delta noted). **The reviewer verified the
load-bearing premise**: nutorchd links by `@rpath/` names and the dylibs carry
`@rpath/` install names, so the third-rpath plan needs no install_name_tool
surgery — and pre-measured the full closure (keep 4 ≈ 208MB / drop 3) for the
Result to confirm independently.

## Result

**Result:** Pass

Nutorch 0.1.0 exists and runs outside its checkout.

- **The adversarial relocation proof, executed**: bootstrap → install to a
  mktemp prefix (211MB, down from 237MB — the measured subset) → `.libtorch` AND
  `.venv-torch` renamed away → from `/tmp` with a private TMPDIR, the installed
  `torch` auto-spawned the installed `nutorchd` via sibling discovery and ran
  `tensor → add → value` = `[4.0, 6.0]` on MPS; `daemon status` showed
  `version: 0.1.0` and `device: mps`; `--json` carried the version. Dev world
  restored; full suite green both before and after.
- **The dylib closure, independently confirmed** (matching the design review's
  pre-measurement): KEEP `libtorch.dylib`, `libtorch_cpu.dylib`, `libc10.dylib`,
  `libomp.dylib` (the non-direct keeper — referenced only by libtorch_cpu as
  `@rpath/libomp.dylib`; its own LC_ID is an absolute `/opt/llvm-openmp` path,
  recorded); DROP `libtorch_python.dylib` (28M), `libshm.dylib`,
  `libtorch_global_deps.dylib`. The subset list lives in `scripts/install.sh`
  with the closure documented inline — the single source Experiment 2's formula
  will mirror.
- **Three rpaths verified baked** (`otool -l`): the two dev paths first, then
  `@loader_path/../libexec/libtorch/lib` — one binary, both worlds, no
  `install_name_tool`.
- **Version story**: workspace-inherited 0.1.0; both binaries print
  `nutorch 0.1.0 (<sha>)` (the daemon BEFORE its MPS gate — GPU-less machines
  can ask); the banner and `status` carry it; build.rs stamps declare
  rerun-if-changed on `.git/HEAD` + the active ref.
- **Bootstrap**: second run skips the download (`already has torch 2.11.0`); the
  fresh-clone simulation (venv symlinked in, per the design-review mechanism)
  bootstraps and passes `cargo test -p nutorch-ops`.
- **Hygiene**: 0 warnings debug AND release; fmt/dprint clean; 79 unit + 255
  golden + smoke + the nutorch.nu staleness test green; `v1/` untouched;
  Cargo.lock regenerated with the bump (the noted delta).

## Conclusion

The substrate holds: one layout, one binary, both worlds. Everything Experiment
2's formula needs now exists — the wheel-pinned bootstrap trick, the measured
dylib subset (in the install script it can mirror), relocatable binaries, and a
version to put in the formula's `url`. Next: the tap repository and the formula,
source-build proven end to end.

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context), reviewing the
pre-commit working tree. **Verdict: APPROVED — no Required, Optional, or Nit
findings.** The reviewer REPRODUCED the adversarial relocation proof itself —
installed to its own temp prefix (211M confirmed), renamed
`.libtorch`/`.venv-torch` away, ran the MPS pipeline from /tmp via
sibling-discovered auto-spawn, then restored the dev world and verified
restoration with git status + tests. It independently confirmed the dylib
closure (3 direct @rpath links; the libomp stowaway via libtorch_cpu; nothing
referencing the dropped three), the three baked rpaths, the version stamp
matching `git rev-parse --short HEAD`, the pre-MPS-gate `--version`, the
rerun-if-changed directives, bootstrap's download-skip, old-client status
compat, and the Cargo.lock delta being exactly the version bumps.
