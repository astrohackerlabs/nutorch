+++
[implementer]
agent = "claude-code"
model = "claude-fable-5"

[review]
waived = "user decision 2026-06-12: no adversarial review for issue 0013"
+++

# Experiment 3: The hero rework — show, don't tell

## Description

Punch-list fix 2, expanded by the user into a full banner reconsideration: the
hero's descriptive paragraph ("nutorch is a tensor daemon for Apple silicon:
PyTorch semantics, Metal GPU compute, and plain string handles that flow through
ordinary pipelines — bash, zsh, fish, Nushell, anything.") is a spec sheet, not
a story. Three copy directions were drafted and offered; the user chose **"Show,
don't tell"**: keep the headline, cut the paragraph to a three-beat line, and
put a real example at the very top so the code does the describing.

**The chosen hero stack (user-approved):**

1. H1 (unchanged): `GPU tensors for every shell`
2. Sub-line:
   `Tensors live on your GPU. Your shell passes handles. Pipes do
   the rest.`
3. The example, right in the hero (the same proven add pipeline, real output):
   ```bash
   a=$(torch tensor '[1,2,3]')
   b=$(torch tensor '[4,5,6]')
   torch add $a $b | torch value
   # [5.0,7.0,9.0]   computed on the GPU
   ```
4. Caption (small, muted):
   `PyTorch semantics on Apple-silicon Metal — from
   bash, zsh, fish, or Nushell.`
5. The Install / GitHub buttons, as today.

**Decisions, made here:**

1. **The logo-left, content-right hero layout survives**; only the right column
   changes (H1 → sub → code → caption → buttons). The code block uses the
   existing `CodeBlock` component (dual Vitesse themes) — nothing new to theme.
2. **The duplicate dies**: the "See it run" section's bash panel currently shows
   the SAME add pipeline. It becomes a different real example —
   `m=$(torch randn '[3,3]')` / `torch mm $m $m | torch mean | torch value` — so
   the page demonstrates more surface instead of repeating itself.
3. **The honesty gate learns about the landing page** (the issue analysis
   promised this): `check-content.ts` gains a scan of `src/pages/index.astro` —
   every `torch <verb>` in the file must be a table op or a known verb, same
   rule as the docs fences. The hero example is also run LIVE during
   verification.
4. **No other sections move**: install block, nushell panel, autograd panel,
   feature cards, footer all stay.

## Changes

1. **`website/src/pages/index.astro`**: the hero stack above; the "See it run"
   bash panel swap.
2. **`website/scripts/check-content.ts`**: index.astro verb scan.
3. **Nothing else** — no Rust, no docs, no `v1/`.

## Verification

1. **Build + gates**: `bun run build` clean; `check:content` (now covering
   index.astro), `check:links`, `check:ops-ref` all green.
2. **The example is true**: run the hero pipeline and the new mm/mean pipeline
   live against the brew binary (private TMPDIR) — outputs match what the page
   shows.
3. **The old paragraph is gone**: grep over `dist/` finds no "tensor daemon for
   Apple silicon" sentence; the three-beat line and caption are present.
4. **Both modes, by eye**: hero screenshots light and dark — the code block sits
   above the fold at a typical viewport, legible, on-brand.
5. **Hygiene**: dprint clean on touched files; `v1/` and the Rust tree
   untouched.

**Pass** = all five. **Fail** = any shown command/output the binary disputes, or
the hero code below the fold at 1440×900.

## Result

**Result:** Pass

The code is the pitch now — visible without scrolling, true to the binary.

- **The hero stack landed as approved**: H1 unchanged; the three-beat line
  ("Tensors live on your GPU. Your shell passes handles. Pipes do the rest.");
  the add pipeline in a Shiki block directly beneath; the caption; the buttons.
  At 1440×900 the ENTIRE stack plus the start of the install section sits above
  the fold in both modes (`logs/issue-0013/hero-{light,dark}.png`).
- **The spec-sheet paragraph is gone everywhere** — including its second life as
  the META DESCRIPTION, which the dist grep caught ("tensor daemon" hits: 1 →
  0); the og:description now matches the new voice ("GPU tensors for every shell
  — PyTorch semantics on Apple-silicon Metal, piped through bash, zsh, fish, or
  Nushell.").
- **The duplicate died**: "See it run"'s bash panel now shows the randn/mm/mean
  pipeline instead of repeating the hero's add example.
- **Both examples run live** against the brew binary before being shown:
  `[5.0,7.0,9.0]` for the hero; the mm/mean pipeline returns a scalar as
  depicted.
- **The honesty gate now covers the landing page**: `check-content.ts` scans
  index.astro's template LITERALS with the docs verb rule — scoped to backtick
  strings after the whole-file first draft false-positived on the hero image's
  alt text ("The nutorch logo: …" → verb "logo"); the scoped scan keeps prose
  out and code in.
- **Gates**: build clean (20 pages), `check:content`/`check:links`/
  `check:ops-ref` green; dprint clean; no Rust changes; `v1/` untouched.

## Conclusion

Fix 2 and the banner reconsideration are done together: the first thing a
visitor sees is tensors being added on a GPU from a shell prompt, with one plain
sentence on either side. The checker false-positive was a good reminder that
prose and code need different rules — the literal-scoped scan is the right
boundary. The punch list is now clear pending new items from the user.
