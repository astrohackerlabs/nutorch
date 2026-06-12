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

# Experiment 2: The docs straighten — both forms, both shells

## Description

With the module now honoring the Dual Input Pattern, the documentation that
issue 0015 had to bend for honesty straightens back out. The dual-input nu
panels get their argument form, and every "pipeline-first by design" hedge
retires. After this the issue closes.

**The inventory** (every place the old limitation leaks into prose or panels):

1. **getting-started, dual-input section**: the nu panel currently shows one
   form with a "pipeline-first" comment; it becomes the two-form twin of its
   bash panel (`$a | nutorch add $b` pipeline / `nutorch add $a $b` argument),
   and the section prose returns to the strong claim — both forms work, in both
   shells.
2. **tensors, dual-input section**: same treatment; the section prose ("the
   Nushell module is pipeline-first by design") rewritten to state the rule once
   for both shells: the leftmost tensor comes from the pipe/stdin or as an
   argument — same grammar, owned by the CLI.
3. **nushell page**: "Wrappers are pipeline-first — the first tensor slot is
   `$in`" updated to describe dual input (pipe `$in` or pass handles as
   arguments; the CLI's stdin-prefix grammar fills the leftmost missing slots in
   both shells).
4. **Nothing else changes**: tab-group counts stay identical on every page
   (panels change content, not pairing), so the `check:tabs` count map is
   untouched; all other twins already use pipeline form, which remains valid.
   (Reviewed and already correct: ops.md's dual-input sentence is shell-neutral
   and honest — no change needed; the five `pipeline-first` hedges enumerated by
   the reviewer are the complete set.)

**Decisions:**

1. **Every displayed nu form runs live first** (explicit-`use`, private TMPDIR)
   — the argument forms verbatim as displayed, even though the parity harness
   already covers the ops, because the harness tests ops and the docs display
   SNIPPETS.
2. **The prose states the rule once, shell-neutrally**, instead of per-shell
   carve-outs — that is the whole point of the issue.

## Changes

1. **`website/src/content/docs/getting-started.md`**: dual-input nu panel +
   section prose.
2. **`website/src/content/docs/tensors.md`**: same.
3. **`website/src/content/docs/nushell.md`**: the wrapper-description sentence.
4. **Nothing else** — no Rust, no module, no plugin/gates, no `v1/`.

## Verification

1. **Displayed snippets reproduce live** (both new nu panels, verbatim).
2. **Build + gates**: `bun run build`, `check:content`, `check:tabs` (count map
   unchanged — asserts the panels changed content, not structure),
   `check:links`, `check:theme` green; dprint clean; zero `.rs` diffs.
3. **The hedge is gone**: grep over `website/src` finds no "pipeline-first"
   remnant (the phrase only ever existed as the limitation's hedge).
4. **By eye**: the getting-started dual-input group screenshotted on its nu tab,
   both modes.

**Pass** = all four. **Fail** = any displayed snippet the module rejects, any
count-map drift, or a surviving hedge.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**Verdict: APPROVED (first pass).** The reviewer grepped every `pipeline-first`
occurrence in `website/src` — exactly five, all issue-0015 hedges, all covered
by the inventory (two prose + two fence comments + the nushell page sentence);
confirmed no legitimate use survives that the grep gate would wrongly delete;
confirmed the count map counts groups (not lines) so the panels' new content
cannot drift it; confirmed the new argument-form snippets pass the honesty
checker's verb scan; and confirmed ops.md's existing dual-input sentence is
already shell-neutral. One Nit folded: that ops.md fact is now recorded in the
inventory for completeness.
