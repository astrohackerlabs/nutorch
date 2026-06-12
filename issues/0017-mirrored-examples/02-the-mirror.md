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

# Experiment 2: The mirror — every pair, line for line

## Description

All fifteen existing tab groups (hero + fourteen docs pairs across six pages)
are rewritten so the nu panel is a line-for-line translation of the bash panel:
same structure, same forms (argument mirrors argument, pipe mirrors pipe), same
comments. Enabled by experiment 1 (`tensor`/`value` argument forms) and issue
0016 (everything else).

**The mirroring rules** (applied uniformly, recorded once):

1. **Capture mirrors capture**: `a=$(torch …)` ↔ `let a = (nutorch …)`.
2. **Argument form mirrors argument form**: `torch add $a $b` ↔
   `nutorch add $a $b`; tensor creation by argument both sides
   (`torch tensor '[1,2,3]'` ↔ `nutorch tensor [1 2 3]`).
3. **`print` is the mirror of "this line writes to stdout"**: where a bash
   line's purpose is displaying a value mid-block, the nu line wraps in
   `print (…)` — semantic mirroring beats token mirroring (a bare nu pipeline
   mid-script displays nothing; the issue-0015 lesson). Line counts stay equal.
4. **Shell-specific lines get a same-line-count nu equivalent that teaches the
   nu way**: bash's `torch tensors --json` line mirrors as a native filter line
   (`nutorch tensors | where bytes > 1mb`) with a comment saying why no JSON is
   needed; `cat handles.txt |` mirrors as `open handles.txt |`.
5. **The dual-input sections show both forms in BOTH panels in the SAME order**
   (argument first, pipe second — bash's current order).
6. **`--json` lines mirror DIRECTLY where the module supports them** (review
   catch): the nn-building pair's `torch nn info $m --json` mirrors as
   `nutorch nn info $m --json` (a real passthrough returning structured data) —
   rule 4's "no JSON needed" rationale applies only where nu's NATIVE return
   already is the structured form (`tensors`, `ops`).
7. **Shell-forced structural divergences are DOCUMENTED EXCEPTIONS, not silent
   drift** (review catch): nu requires a `mut loss = ""` declaration line before
   a loop whose binding must outlive it — bash has no peer line; and comment
   re-wrapping can split one bash line across two physical lines. The committed
   audit (below) compares COMMAND lines (binding/invocation lines; blank and
   comment-only lines excluded) and carries an explicit per-pair exception list
   for the mut-declaration class. Inside loop bodies, capture mirrors capture
   (bash `pred=…` / `loss=…` stay TWO lines in nu — no merging).
8. **The landing page's "See it run" side-by-side and the bash-only autograd
   demo stay as they are** (review catch, recorded with rationale): the
   side-by-side was explicitly user-approved as-is in the issue-0015 decisions
   ("the homepage 'See it run' side-by-side — user: fine as-is"), and
   single-panel demos are not pairs; mirroring scope is PAIRED examples. If the
   user wants these reworked, that is a new instruction.

**Verification follows the established bar**: every rewritten nu panel runs live
verbatim (explicit-`use`, private TMPDIR); displayed outputs must reproduce (the
training loop's seeded `2.46e-7`; the hero's `[5.0, 7.0, 9.0]`); the bash
panels' changed lines (if any) run live via the CLI. Specific live questions to
settle during implementation, recorded in the result: whether
`nutorch nn zero_grad $opt` needs `| ignore` inside a `for` body, and whether
`open handles.txt | nutorch add $b` feeds the handle correctly (a real file in
the harness).

## Changes

1. **`website/src/pages/index.astro`**: the hero nu demo mirrors
   (tensor-by-argument, add-by-argument). "See it run" and the autograd demo
   stay (rule 8). 1b. **`website/scripts/check-mirror.ts`** (NEW) +
   `check:mirror` package script.
2. **`website/src/content/docs/{getting-started,daemon,tensors,ops,autograd,neural-networks}.md`**:
   all fourteen docs pairs mirrored per the rules.
3. **Nothing else** — no Rust, no module, no plugin, no count-map changes
   (pairings unchanged), no `v1/`.

## Verification

1. **Every rewritten nu panel reproduces live, verbatim** (and any changed bash
   lines via the CLI); the training loop prints the displayed value.
2. **Structure unchanged**: `check:tabs` count map green as-is (content changed,
   pairing did not); `check:content` green (all fences scanned).
3. **Mirror audit, COMMITTED as a gate** (review catch — the house pattern
   commits its gates): `website/scripts/check-mirror.ts` (`check:mirror`)
   extracts every bash+nu pair from the docs markdown AND the hero's template
   literals, compares command-line counts per pair (blank and comment-only lines
   excluded) modulo the documented exception list, and fails on drift. "Forms
   aligned" is the by-eye criterion (screenshots), not a mechanical claim of the
   gate.
4. **Gates**: build, `check:links`, `check:theme`, brand gate, dprint; zero
   `.rs` diffs.
5. **By eye**: hero + one docs page, nu tab, both modes.

**Pass** = all five. **Fail** = any panel the shell rejects, any displayed
output that does not reproduce, or any pair with unequal line counts.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**First pass: CHANGES REQUIRED** — 3 Required: the nn-building pair's `--json`
line had no mirroring rule (folded as rule 6 — mirror directly via the real
`nutorch nn info --json` passthrough; rule 4's "no JSON needed" rationale scoped
to verbs whose native return is already structured); shell-forced structural
divergences (nu's `mut` declaration before a loop, bash comment wrapping) made
the equal-counts gate mechanically unreachable (folded as rule 7 — the gate
counts COMMAND lines, excludes comment-only and blank lines, carries a
documented exception list for the mut-declaration class, and forbids loop-body
merging); the landing page's "See it run" side-by-side and bash-only autograd
demo were unaddressed (folded as rule 8 — recorded out with the issue-0015
user-approval citation and the pairs-only scope rationale). Optionals folded:
the mirror audit is COMMITTED as `check:mirror` (the house pattern), and "forms
aligned" is the by-eye criterion, not a mechanical claim. **Second pass:
APPROVED** — all five folds verified coherent, and the revised gate confirmed
mechanically reachable for both pairs the first pass named (the training pair
equalizes under the mut exception at 14=14+1-exempt; the daemon pair equalizes
at 5=5 once comment-only lines are excluded).
