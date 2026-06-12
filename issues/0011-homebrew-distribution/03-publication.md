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

# Experiment 3: Publication — the tap goes live and pours a bottle

## Description

The issue's outward-facing finale. Every external action is enumerated here, all
within the issue's user-approved scope ("the tap repository with the formula";
"bottling to a GitHub Release"):

1. **Push `main`** to `github.com/nutorch/nutorch` (6 commits ahead — verified;
   the remote already tracks this work) plus this experiment's plan commit.
2. **Tag and push `v0.1.0`** (the version Experiment 1 stamped). The tag points
   at this experiment's plan commit — the publication acts on exactly the
   reviewed state. Known and intentional: the tagged tree still contains the
   `file://` form of `dist/nutorch.rb`; harmless, because brew reads the TAP's
   formula, never the one inside the source tarball.
3. **One Release `v0.1.0` on `nutorch/nutorch`** carrying the SOURCE tarball
   (`make-source-tarball.sh` output, `git archive` of the tagged commit) as an
   immutable asset. The formula's `url` points at this asset — NOT at GitHub's
   auto-generated `/archive/refs/tags/` tarball, whose checksums have
   historically churned (the Jan-2023 incident). Release assets are
   upload-once-immutable; the vendoring premise stays sound.
4. **Create the public repo `nutorch/homebrew-nutorch`** — by pushing the LOCAL
   tap Experiment 2 already created (`brew tap-new`'s skeleton at
   `Library/Taps/nutorch/homebrew-nutorch` becomes the working copy:
   `gh repo create --source … --push`). The dress rehearsal's stage becomes the
   venue.
5. **One Release `nutorch-0.1.0` on the tap repo** carrying the bottle asset.

`gh` is authenticated (account `ryanxcharles`, SSH) — verified.

**Decisions, made here:**

1. **The brew-6.0 trust gate is part of the product surface** (design-review
   catch, verified live): `HOMEBREW_REQUIRE_TAP_TRUST` is SET by default in brew
   6.0.0, and brew refuses to load formulae from untrusted third-party taps. The
   honest install is therefore THREE commands —
   `brew tap nutorch/nutorch && brew trust nutorch/nutorch &&
   brew install nutorch`
   — documented identically in the tap README and this repo's README.
   Verification runs the cold path with trust REVOKED first
   (`brew untrust nutorch/nutorch` — the first-class command), proving both the
   refusal and the post-trust pour.
2. **The published formula** is `dist/nutorch.rb` with exactly the documented
   publication edits: `url` →
   `https://github.com/nutorch/nutorch/releases/download/v0.1.0/nutorch-0.1.0.tar.gz`,
   `sha256` → the uploaded asset's hash (the local tarball's, verified equal to
   the downloaded asset's). `dist/nutorch.rb` in THIS repo is updated to match
   in the result commit — INCLUDING its header comment, which still promises the
   now-rejected `/archive/` URL (one source of truth maintained; its file://
   twin retired — the hermetic form is reproducible from git history).
   `make-source-tarball.sh` is retargeted in the same commit: once the formula
   points at the immutable Release asset, the script's patch-sha-into-formula
   step is obsolete and is removed — the script keeps building the tarball and
   becomes the release-asset builder. The sha chicken-and-egg from Experiment 2
   is GONE: the tap's formula does not live inside the tarball it describes.
3. **The strict order** (no published artifact may reference one that does not
   exist yet): push main (incl. plan commit) → tag v0.1.0 → push tag → build
   tarball from the tag → create Release v0.1.0 on nutorch/nutorch + upload
   tarball → compute sha → write tap formula (asset url + sha) → commit in the
   local tap → push tap repo into existence (`gh repo create --source`) →
   `brew install --build-bottle nutorch/nutorch/nutorch` (downloads the asset;
   proves url+sha) →
   `brew bottle --json --no-rebuild
   --root-url=https://github.com/nutorch/homebrew-nutorch/releases/download/nutorch-0.1.0`
   → create Release `nutorch-0.1.0` on the tap repo + upload bottle → merge the
   bottle block into the tap formula (manual edit from the emitted JSON — not
   `--merge --write`, which auto-commits; the tap commit is made deliberately) →
   push tap → cold-user verification.
4. **The double-dash gotcha, named**: `brew bottle` writes
   `nutorch--0.1.0.<tag>.bottle.tar.gz` locally but the asset must be uploaded
   as `nutorch-0.1.0.<tag>.bottle.tar.gz` (single dash) — renamed at
   `gh release upload` time. The macOS bottle tag is whatever brew emits on this
   machine (Darwin 25 → recorded in the Result), and the emitted `cellar` value
   is recorded too — with all three rpaths `@loader_path`-relative the
   expectation is `cellar :any` or `:any_skip_relocation` (no Cellar-absolute
   paths to rewrite); one bottle, one platform — honest for a personal-machine
   tap (more platforms are CI work, recorded as the known follow-up).
5. **Gatekeeper (issue design question 5), answered empirically**: the poured
   bottle's binaries are exercised end to end; brew strips quarantine on pour,
   and ad-hoc signatures survive our pipeline (the relocation tests never
   re-signed) — observed, recorded either way (`codesign -dv` + `xattr -l`
   output).
6. **The final UX is the acceptance**: from the user's perspective, untap
   everything local, revoke trust, clear brew's download cache of nutorch
   artifacts (so the bottle genuinely downloads from the Release), then cold:
   tap → trust → install → the bottle pours → MPS pipeline works.

## Changes

1. **`github.com/nutorch/nutorch`**: main pushed; tag `v0.1.0`; Release `v0.1.0`
   with the source-tarball asset.
2. **`nutorch/homebrew-nutorch`** (NEW public repo): `Formula/nutorch.rb`
   (release-asset URL + bottle block), README with the three-line install (tap,
   trust, install); Release `nutorch-0.1.0` with the bottle asset.
3. **THIS repo** (the result commit): `dist/nutorch.rb` updated to the published
   form; README's install section leads with brew (three commands, trust gate
   explained in one sentence); the issue docs.

## Verification

1. **The cold-user path** (the acceptance): untap/uninstall everything local,
   revoke trust, purge cached nutorch downloads → `brew tap nutorch/nutorch`
   (from GitHub) → `brew install nutorch` REFUSES (the trust gate, observed) →
   `brew trust nutorch/nutorch` → `brew install nutorch` pours the BOTTLE —
   asserted concretely: install output contains `Pouring
   nutorch-0.1.0`, no
   `cargo`/`Building` lines, `rust` not installed as a dependency, wall-clock
   well under the ~32s source build → `torch --version` = 0.1.0 → MPS pipeline
   `[1,2]+[3,4] = [4.0,6.0]` with a private TMPDIR → `brew test nutorch` green.
2. **Source-build fallback intact**:
   `brew install --build-from-source
   nutorch` also works — proving the
   release-asset url+sha for users on a future macOS with no bottle. (The
   `--build-bottle` install in the publication sequence already exercises this
   download path once.)
3. **Gatekeeper**: the poured binaries run without quarantine prompts;
   `codesign -dv` and `xattr -l` output recorded.
4. **Hygiene**: this repo's suite untouched and green; dprint/fmt clean on
   touched files; `v1/` untouched.

**Pass** = all four. **Fail** = the bottle does not pour cold, or the uploaded
asset's sha drifts from the local tarball's (would indicate the asset pipeline
is not the immutable store the design claims).

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**First pass: CHANGES REQUIRED** — 1 Required: the brew-6.0 **tap trust gate**
was unaddressed — `HOMEBREW_REQUIRE_TAP_TRUST` is SET by default in brew 6.0.0
(verified live: `brew list nutorch` refuses to load the untrusted tap), so the
two-command install was a flow the verification machine itself would refuse.
Absorbed: the honest install is three commands (tap, trust, install), documented
in both READMEs, and verification now proves the refusal AND the post-trust
pour. Optionals folded: the source `url` retargeted from GitHub's auto-generated
`/archive/` tarball (checksum-churn history) to an immutable Release asset built
by `make-source-tarball.sh`; the strict publication order pinned (no forward
references — verified link by link in the second pass); the bottle-pour
assertions made concrete (`Pouring` line, no cargo/Building, no rust dep,
wall-clock); the emitted `cellar` value recorded. Nits folded: the tag pins the
plan commit and the stale `file://` formula inside the tarball is declared
intentional; trust revocation uses first-class `brew untrust`. **Second pass:
APPROVED** — the reviewer independently reproduced the trust gate, traced the
publication order for forward references (none), and confirmed the
chicken-and-egg is structurally gone. One Optional folded
(`make-source-tarball.sh`'s sha-patch step becomes obsolete post-retarget —
removed in the result commit) and one Nit (the `dist/nutorch.rb` header comment
still promising the `/archive/` URL must be rewritten with it).

## Result

**Result:** Pass

`brew tap nutorch/nutorch && brew trust nutorch/nutorch && brew install
nutorch`
is live. The bottle pours from GitHub in **5.4 seconds**.

**The publication, in the pinned order — every step as designed:**

- main pushed (`ec32998..9832011`, the plan commit at the tip); tag `v0.1.0` cut
  on the plan commit and pushed.
- Release `v0.1.0` on `nutorch/nutorch` with the source tarball built by the
  retargeted `make-source-tarball.sh` from the tag; sha
  `a50d68d4ec770b0b742cf07e89c9947597b7989d6be9597c8f183a17bbfd6c21`; the
  downloaded asset re-hashed BYTE-IDENTICAL (the immutability check).
- The local tap from Experiment 2 became the venue: formula (release-asset url +
  sha, `version` line dropped — brew reads 0.1.0 off the asset name) and a
  README with the three-command install committed (`f6c6346`), then
  `gh repo create nutorch/homebrew-nutorch --source . --push`. `tap-new`'s stock
  CI workflows kept (push runs tap-syntax only — safe).
- `brew install --build-bottle nutorch/nutorch/nutorch` built from the LIVE
  release asset in 31s (proving url+sha in the act of bottling);
  `brew bottle --json --no-rebuild --root-url=…` emitted
  `nutorch--0.1.0.arm64_tahoe.bottle.tar.gz` (47.9MB) — uploaded single-dash to
  Release `nutorch-0.1.0` on the tap repo (the double-dash rename, as scripted);
  bottle block merged manually, tap pushed (`b31c7fb`).
- **The bottle tag is `arm64_tahoe`** (macOS 26.5). **The emitted block has NO
  `cellar` line** (default `/opt/homebrew/Cellar`), and the bottle JSON's
  `changed_files` is **empty** — zero install-name rewrites at bottling, the
  `@loader_path` rpath design earning its keep one last time.

**The cold-user acceptance (verification 1):** uninstall + untap + untrust +
cache purged → `brew tap nutorch/nutorch` cloned from GitHub →
`brew install nutorch` **REFUSED** (the trust gate, observed exactly as the
design review predicted) → `brew trust nutorch/nutorch` → `brew install nutorch`
→ output `Pouring
nutorch-0.1.0.arm64_tahoe.bottle.tar.gz`, **5.41s wall**, no
cargo/Building lines, rust NOT installed → `torch --version` =
`nutorch 0.1.0 (unknown)` → MPS pipeline `[1,2]+[3,4] = [4.0,6.0]` with a
private TMPDIR, daemon status `device: mps`, version 0.1.0 → `brew test nutorch`
green.

**Source-build fallback (verification 2):**
`brew install
--build-from-source nutorch` from the same release asset: 45.9s
including dependency pours — works; the bottle install restored afterward.

**Gatekeeper (verification 3, issue design question 5 answered):** poured
binaries are `Signature=adhoc` `linker-signed`; `xattr -l` is EMPTY on both the
binary and the dylibs (brew strips quarantine on pour); everything ran with zero
prompts.

**Hygiene (verification 4):** build 0 warnings, `cargo fmt --check` clean, all
suites green (3+79+1+3+1+1 tests), dprint clean on touched files (pre-existing
unformatted files under frozen `v1/` and `docs/archive/` are outside the
touched-files gate), `v1/` untouched.

Repo-side result edits: `dist/nutorch.rb` to the published form (header comment
rewritten — the `/archive/` promise excised), `make-source-tarball.sh`
retargeted as the release-asset builder (sha-patch step removed), README's
install section now leads with the three brew commands.

## Conclusion

Nutorch is distributed. The full path — clone-free install, trust gate and all —
is three commands and ~6 seconds on a bottled platform, with a working source
build behind it for unbottled macOS versions. The vendoring chain is immutable
end to end: PyPI wheel (hash-pinned) → release-asset tarball (hash-pinned,
upload-once) → bottle (hash-pinned, upload-once). What remains is operational,
not architectural: bottles for more macOS versions (CI), and a release-cutting
script to mechanize the order this experiment executed by hand. The issue's goal
is met; close it.

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context), reviewing BEFORE
the result commit (the gate ordering restored after Experiment 2's lapse — the
reviewer confirmed the working tree held the uncommitted result). **Verdict:
APPROVED — no Required findings.** The reviewer independently re-verified every
load-bearing claim against the live artifacts: both release assets downloaded
and re-hashed to exact sha matches; the tap formula equals `dist/nutorch.rb`
plus only the bottle block, with url/sha/root_url/bottle-sha all
self-consistent; version inference from the asset name confirmed via `brew list`
and `brew test`; the poured keg's MPS pipeline, adhoc signature, empty xattrs,
and absent rust all reproduced; the trust step present in both READMEs;
fmt/dprint clean. The reviewer attempted to break the formula consistency, sha
pins, version inference, bottle naming, and process ordering — each held. **One
Nit, acknowledged here**: "release assets are upload-once-immutable" overstates
GitHub's guarantee (the releases report `immutable: false`); the operative
integrity guarantee is the formula's sha256 pin — a mutated asset fails the
hash. The design phrase describes workflow practice, not platform enforcement.
