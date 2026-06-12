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

# Experiment 1: The mechanism — paired fences become synced tabs

## Description

The machinery, proven end to end on one real example before any mass content
work (Experiment 2 writes the remaining Nushell twins): the rehype plugin that
pairs adjacent `bash`+`nu` fences into tab groups, the ONE shared script that
drives every group plus the hero, the unified `shell` storage key, and a
committed CDP gate for the sync behavior.

**Decisions, made here:**

1. **The plugin** is a local file, `website/plugins/rehype-shell-tabs.mjs`,
   registered in `astro.config.mjs` `markdown.rehypePlugins` — zero new
   dependencies (a hand-rolled tree walk; no unified helper packages). It runs
   AFTER Astro's Shiki step and looks for a `<pre data-language="bash">` whose
   next ELEMENT sibling (ignoring whitespace-only text nodes — markdown output
   interleaves `"\n"` text nodes between blocks) is `<pre data-language="nu">`,
   replacing the pair with the tab-group wrapper: `role="tablist"` buttons + two
   panels, nu panel `hidden`, ids suffixed by a per-page counter so
   `aria-controls` stays unique with many groups on one page. Unpaired fences
   pass through byte-identically. The authoring contract is ORDERED: bash fence
   first, nu fence immediately after (nu-then-bash stays un-paired — documented,
   review nit). (The `data-language` attribute's presence on built `<pre>`
   elements is verified against dist before the plugin relies on it — the
   reviewer already confirmed both the attribute and the whitespace text nodes
   between siblings in current dist output.)
2. **Tab styling moves into plain CSS component classes** (`.shell-tab`,
   `.shell-tab[aria-selected="true"]`, `.shell-tabs` layout) in `global.css`,
   replacing the hero buttons' inline Tailwind utility strings. Reason: the
   plugin injects markup at build time from a file Tailwind does NOT scan — if
   the plugin used utility classes that only the hero happened to share, editing
   the hero could silently unstyle every docs tab group. Component classes kill
   that coupling.
3. **One script, in `Base.astro`**, drives every group via event delegation: the
   stored preference is resolved and applied in the `is:inline` HEAD script
   alongside the theme init (review catch — a deferred apply would flash bash at
   a returning Nushell visitor; the head script sets a `data-shell` attribute on
   `:root` that CSS uses to show/hide panels before first paint, with the
   `hidden` attributes kept in sync by the runtime script for assistive tech);
   one delegated click listener flips ALL groups and persists. With no
   JavaScript at all, `data-shell` is never set, so the static `hidden`
   attributes govern and a no-JS visitor sees the bash default (second-pass
   review nit, stated explicitly). The hero's per-component script is DELETED
   and its markup adjusted to the same data-attribute pattern (its `CodeBlock`
   panels stay — only the wiring changes).
4. **The storage key unifies as `shell`** (`posix`/`nu`), default `posix`.
   Migration: if `shell` is absent and legacy `hero-shell` exists, its value is
   adopted and the legacy key removed.
5. **TWO real paired examples ship with the mechanism** (review catch: with one
   group per page, "all groups flip together" would be vacuously true and the
   Fail clause untestable): getting-started's "First tensors" AND "A taste of
   more" blocks both get Nushell twins (each run live first via the
   discriminating explicit-`use` form — the issue-0014 rule cited in the spine;
   outputs must match the displayed comments). Two groups on one page make the
   same-page simultaneous flip a REAL assertion; the remaining twins are
   Experiment 2.
6. **The CDP gate is committed** as `scripts/check-shell-tabs.ts` (`check:tabs`
   script), reusing the check-theme harness shape (isolated user-data-dir,
   endpoint polling). Matrix: fresh visit → bash everywhere, no stored key;
   click Nushell on the FIRST docs group → BOTH groups on the page flip in the
   same action (the real multi-group assertion), `shell=nu` stored; navigate to
   the homepage → the hero shows Nushell; legacy-key migration (`hero-shell=nu`,
   no `shell`) resolves to nu and rewrites the keys; click back → `posix`
   stored, all groups flip back.

## Changes

1. **`website/plugins/rehype-shell-tabs.mjs`** (NEW) + **`astro.config.mjs`**
   registration.
2. **`website/src/styles/global.css`**: `.shell-tabs`/`.shell-tab` component
   classes (hero utilities folded in).
3. **`website/src/layouts/Base.astro`**: the shared script (+ key migration).
4. **`website/src/pages/index.astro`**: hero markup on the shared pattern; its
   local script removed.
5. **`website/src/content/docs/getting-started.md`**: TWO paired examples (nu
   twins under the "First tensors" and "A taste of more" bash fences).
6. **`website/scripts/check-shell-tabs.ts`** (NEW) + `check:tabs` package
   script.
7. **No Rust; no `v1/`; no other content (Experiment 2).**

## Verification

1. **The twins reproduce first** (the gate that stops everything if it fails):
   both displayed nu snippets run via
   `nu -c 'use /opt/homebrew/share/nutorch/nutorch.nu *; …'` and match their
   displayed outputs.
2. **Build**: clean; the getting-started page emits exactly TWO tab groups
   (asserted: two `role="tablist"`, four panels, nu panels hidden, all ids
   unique). The no-touch proof is a FENCE-LEVEL baseline diff (second-pass
   review catch — a whole-page byte diff cannot work because this experiment
   also changes `global.css`, whose hashed `<link>` appears on every page, and
   the shared head script): the BASELINE is the IDENTICAL final source tree with
   only the rehype plugin unregistered (third-pass catch — against
   pre-experiment main, getting-started's new nu fences would make identity
   impossible; this baseline isolates the plugin as the sole delta). Extract the
   ordered list of `<pre …>…</pre>` blocks from EVERY page in both builds and
   assert the lists are identical for every page — including getting-started
   (the plugin wraps and MOVES the paired `<pre>` nodes but must not alter their
   bytes) and index.html (the hero's `CodeBlock` pres are rewired around, not
   through). This is the direct form of the claim "no fence content changed
   anywhere."
3. **`check:tabs` green**, and adversarially: with the plugin handed a page of
   only unpaired fences, zero wrappers appear (the nushell.md page IS that page
   — asserted).
4. **Existing gates**: `check:content` (the new nu fence is scanned
   automatically), `check:links`, `check:ops-ref`, `check:theme`, brand gate —
   all green; dprint clean; zero `.rs` diffs.
5. **Both modes by screenshot**: the getting-started tab group, bash and nu
   tabs, light and dark — styling matches the hero control.

**Pass** = all five. **Fail** = either twin does not reproduce, any unpaired
fence changes, or the same-page flip leaves either group behind (now a real
two-group assertion).

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only + local
builds). **First pass: CHANGES REQUIRED** — 1 Required: the headline promise
("all groups on a page flip together") was UNTESTABLE as designed — with only
one paired example on the test page, "every group flips" is vacuous and the Fail
clause ("leaves any group behind") could never fire. Absorbed: getting-started
ships TWO paired examples ("First tensors" and "A taste of more"), making the
same-page simultaneous flip a real two-group assertion in the CDP matrix.
Optionals folded: the no-touch proof is now a precise baseline diff (every page
byte-identical except index.html and getting-started); the stored shell is
applied in the `is:inline` HEAD script to avoid flashing bash at a returning
Nushell visitor (the theme init's pre-paint pattern). Nit folded: the authoring
contract is ordered — bash fence first, nu immediately after. The reviewer
independently confirmed the load-bearing premises: built `<pre>` elements carry
`data-language` with whitespace text nodes between siblings; user rehypePlugins
run AFTER Shiki (verified in `@astrojs/markdown-remark` source, so the plugin
sees highlighted output); the Tailwind-scanning hazard is real and the
component-class mitigation matches the established `hero-shell-tab` CSS pattern;
and no other docs page carries an adjacent bash→nu pair, so the baseline diff is
clean.

**Second pass: CHANGES REQUIRED** — 1 Required: the baseline-diff method the
first pass suggested was itself incoherent with the experiment's own global
changes (the hashed `global.css` link and the shared head script appear on EVERY
page, so whole-page byte identity outside two exceptions could never hold).
Absorbed: the no-touch proof is now FENCE-LEVEL — the ordered `<pre>` blocks of
every page, baseline vs new, must be identical everywhere, including the wrapped
pages (the plugin moves nodes, never alters their bytes), which is the direct
form of the actual claim. Nit folded: the no-JS outcome stated explicitly (no
`data-shell` attribute → static `hidden` governs → bash default). **Third pass:
APPROVED** — fence-level method confirmed coherent; one Optional folded: the
baseline is the identical source with only the plugin unregistered, isolating
the plugin as the sole delta.

## Result

**Result:** Pass

Paired fences become synced tabs, and every claim in the design held — after one
build-system surprise.

- **The twins reproduced first**: both nu snippets ran via the discriminating
  explicit-`use` form before being written into the page (`[5.0, 7.0, 9.0]`; the
  mm/mean scalar; the gradient vector — and the module flag is
  `--requires_grad`, underscore, matching the displayed code).
- **The plugin works as designed** — but the first build LIED: Astro's content
  layer caches rendered markdown in `node_modules/.astro` (not just `.astro/`),
  so the new plugin appeared to do nothing until BOTH cache locations were
  cleared. The result reviewer then proved the hazard was still LIVE (a
  stale-cache build with the plugin disabled kept serving tabs — a green
  false-positive factory), so the lesson is now ENFORCED IN CODE:
  `bun run build` cleans both cache dirs first (a full 20-page re-render costs
  ~1s). Adversarially re-proven: a disabled build right after a normal one emits
  zero tab groups.
- **Structure asserted**: getting-started emits exactly 2 tablists / 4 panels /
  2 hidden nu panels / 8 unique ids; the nushell page (all unpaired fences)
  gains zero wrappers.
- **The fence-level baseline diff holds everywhere**: with the identical tree
  built once with `SHELL_TABS_DISABLE=1` and once live (caches cleared both
  sides), the ordered `<pre>` lists of ALL 20 pages are identical — the plugin
  moves nodes, never alters bytes.
- **The CDP matrix (`check:tabs`) is all green**: fresh visit → bash everywhere,
  nothing stored; ONE click flips BOTH docs groups in the same action;
  `shell=nu` stored; the HERO follows on the homepage (cross-page, shared key);
  click back → `posix`; legacy `hero-shell=nu` migrates to `shell=nu` with the
  old key removed.
- **Pre-paint proven**: a page served with `data-shell="nu"` pre-set renders nu
  panels and active nu tabs from CSS alone
  (`logs/issue-0015/tabs-{bash,nu}-{light,dark}.png` — all four states, on-brand
  in both modes).
- **Gates**: `check:content` (the twins scanned automatically), `check:links`,
  `check:ops-ref`, `check:theme`, the brand gate — all green; dprint clean; zero
  `.rs` diffs; `v1/` untouched.

## Conclusion

The mechanism is real: authors write two adjacent fences and the site does the
rest — one plugin, one script, one preference, pre-paint correct, no-JS safe.
The cache discovery (`node_modules/.astro`) is the lesson worth remembering.
Experiment 2 is now pure content: Nushell twins for the remaining docs examples,
each judged against the exemption list and verified live before display.

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context), reviewing BEFORE
the result commit. **First pass: CHANGES REQUIRED** — 1 Required: the cache
lesson lived only in prose — the reviewer demonstrated that with a populated
cache, a build with the plugin DISABLED still served tab markup, meaning the
whole gate suite could certify stale HTML. Fixed in code: the `build` script now
removes `node_modules/.astro` and `.astro` before every build, and the
adversarial disabled-build check now yields zero tab groups. 1 Optional: the
second nu twin's mean line printed nothing when the fence ran as a script
(Nushell auto-prints only the last pipeline), breaking output parity with its
bash twin — fixed with an explicit `print`, and the exact shipped snippet
re-verified live (scalar AND gradient vector both print). 1 Nit: the legacy
`hero-shell` key now gets removed unconditionally (previously a visitor with
`shell` already set kept the stale key forever). The reviewer independently
reproduced everything else: both twins, the 20-page fence-level identity, all
five structural assertions, the hero migration with `hero-shell` surviving only
in the migration code, the 7/7 CDP matrix, all gates, the screenshots, and the
process state (plan commit a3d2ba2 plan-only; zero `.rs` diffs; `v1/`
untouched).

**Second pass: APPROVED** — the reviewer reproduced all three fixes: the
adversarial disabled-build now emits zero tab groups (the stale-cache
false-positive factory is dead, enforced by the build script); the exact shipped
second twin prints BOTH results; the legacy key is removed unconditionally with
the 7/7 CDP matrix still green.
