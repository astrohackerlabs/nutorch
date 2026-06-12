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

# Experiment 1: The symlink

## Description

`nutorch` becomes a symlink to `torch` wherever the binaries are installed. One
experiment, three install paths plus docs.

**Decisions, made here:**

1. **`scripts/install.sh`**: after copying the binaries,
   `ln -sf torch "$PREFIX/bin/nutorch"` (relative target — the pair moves
   together); the final verification line also runs
   `"$PREFIX/bin/nutorch" --version`.
2. **`dist/nutorch.rb`**: `bin.install_symlink "torch" => "nutorch"` after the
   existing `bin.install`; the `test do` block gains
   `assert_match "nutorch #{version}", shell_output("#{bin}/nutorch --version")`.
   (The published tap copy is NOT updated — next release; recorded in the issue
   spine.)
3. **The user's live install gains it now**: the brew keg's bin gets the symlink
   locally (`ln -s torch` in the keg, plus the matching link in
   `$(brew --prefix)/bin`) — explicitly recorded as a hand-applied convenience
   that the NEXT RELEASE's `brew upgrade` makes official (the published tap is
   untouched until then). Honesty caveat (review catch): the hand-made prefix
   link is untracked by brew — orphaned by `brew uninstall` and possibly needing
   manual removal before the next release's relink.
4. **Docs touch**: install-from-source page says both names install; the README
   install section gets one clause. No hero/landing changes — `torch` remains
   the canonical name in examples (PyTorch fidelity).
5. **No Rust changes**: nothing reads `argv[0]`; the sibling daemon lookup
   (`current_exe().parent()`) is direction-proof because the link lives in the
   same directory as its target.

## Changes

1. **`scripts/install.sh`**: symlink + verification line.
2. **`dist/nutorch.rb`**: `bin.install_symlink` + test assertion.
3. **`website/src/content/docs/install-from-source.md`** + **`README.md`**: one
   line each.
4. **Local convenience**: keg + brew-prefix symlinks on this machine.
5. **No Rust; no `v1/`; published tap untouched (recorded).**

## Verification

1. **From-source path**: run `scripts/install.sh` into a TEMP prefix;
   `<prefix>/bin/nutorch --version` prints the version;
   `nutorch tensor
   '[1,2]' | nutorch value` works end to end with a private
   TMPDIR (the sibling daemon spawn proves unaffected by the symlink). This is
   the MPS dev-machine gate; the formula's `test do` stays GPU-free via
   `--version`, consistent with the existing block.
2. **Formula**: `brew style` exercised against the NEW line by temporarily
   copying `dist/nutorch.rb` over the local tap's formula copy (uncommitted,
   never pushed), running style, and reverting (review catch — styling the tap's
   old copy would not test the new line); no new offenses; the formula diff is
   exactly the two declared lines.
3. **The live machine**: `which nutorch` resolves in a fresh shell;
   `nutorch daemon status` round-trips against the daemon spawned by `torch`
   (same socket — they are the same binary).
4. **Docs**: website builds clean; `check:content`/`check:links` green; dprint
   clean on touched md.
5. **Hygiene**: no Rust diffs (`git status` proves it); `v1/` untouched.

**Pass** = all five. **Fail** = the symlinked name fails to spawn or find the
daemon, or any installer leaves a dangling link.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**Verdict: APPROVED (first pass).** The reviewer verified the load-bearing
claims against the source: the sibling daemon lookup is genuinely
direction-proof across all three install layouts (from-source prefix, brew keg,
brew-prefix binstubs — every possible `current_exe()` resolution lands in a bin
containing `nutorchd`); nothing in torch-cli reads `argv[0]`; `install.sh`
already takes a prefix argument (gate 1 feasible);
`bin.install_symlink "torch" => "nutorch"` is the correct target=>link DSL and
`brew link` propagates keg symlinks like any binstub. Two Optionals folded: gate
2 now tests the NEW formula line by temporarily copying `dist/nutorch.rb` over
the local tap copy (style against the old copy would prove nothing), and
decision 3's honesty gap closed (the manual prefix link is brew-untracked —
orphaned on uninstall; "next release's upgrade," not "next upgrade," makes it
official). Nit folded: gate 1 named as the MPS dev-machine gate, with the
formula test staying GPU-free.
