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

# Experiment 2: The docs — content collections and the written pages

## Description

The documentation system and the hand-written documentation itself: Astro
content collections under `src/content/docs/`, a `DocPage` layout (sidebar,
prose styling, prev/next), and the full set of written pages — drafted from the
repo README (the proven first draft) and expanded with what the issues record.
The GENERATED ops reference is Experiment 3; search/sitemap/OG finishing is
Experiment 4.

**Decisions, made here:**

1. **Content collections, typed**: `src/content.config.ts` defines a `docs`
   collection (glob loader over `src/content/docs/**/*.md`) with a zod schema:
   `title`, `description`, `order` (sidebar position), optional `section`.
   Markdown only — no MDX dependency until something actually needs a component
   in prose.
2. **Routing**: `src/pages/docs/[...slug].astro` renders every collection entry
   through `DocPage.astro` via `render(entry)` from `astro:content` (the Astro 6
   API — `entry.render()` is removed); `/docs` RENDERS Getting Started directly
   (review catch: no static meta-refresh redirect hop; the duplicate URL pair
   `/docs` + `/docs/getting-started` gets a `<link rel="canonical">` pointing at
   the slug URL). Fenced code blocks get the dual Vitesse themes from
   `markdown.shikiConfig` — already configured in Experiment 1 and waiting; the
   `!important` glue already covers them (same `.astro-code` class).
3. **`DocPage.astro`**: termsurf's shape, nutorch's skin — left sidebar
   (sections + links, current page highlighted with the brand green, hidden on
   mobile behind a `<details>` disclosure), article column with a
   `.prose-nutorch` class styled in `global.css` (headings in Space Grotesk,
   links in primary, inline code chips, tables, blockquotes), and prev/next
   links at the bottom derived from `order`. The sidebar is BUILT FROM THE
   COLLECTION (sections + order from frontmatter), not hardcoded — one less
   thing to forget when adding a page; the ops reference (Exp 3) joins the same
   tree automatically.
4. **The pages** (8, first-class drafts — each opens with what the feature is,
   shows real commands with real output, and links onward):
   - `getting-started.md` — install (brew three commands), first tensors, first
     pipeline, where to go next.
   - `daemon.md` — auto-start, idle TTL + lease renewal,
     `daemon
     status|ttl|stop|restart`, `NUTORCHD_TTL`, the memory-horizon
     contract, socket/log locations.
   - `tensors.md` — handles (`tensor://`), the dual input pattern (stdin prefix
     grammar), creation ops, `value`/`--meta` export-import, the non-finite JSON
     dialect, `tensors`/`free`/`free --all`.
   - `autograd.md` — requires_grad, backward, grad snapshots, zero_grad, detach,
     the rules of the road (scalar loss, rebuild-graph-per-backward, keep leaf
     handles).
   - `neural-networks.md` — `nn://` modules (all 19 kinds listed), sequential
     composition, forward, parameters as live views, optimizers
     (sgd/adam/adamw/rmsprop), train/eval, save/load (safetensors,
     PyTorch-interchangeable), the train-regression walk-through.
   - `nushell.md` — the generated module, `use nutorch.nu *`, structured data
     in/out, non-finite handling, `--json` on the structured verbs, the training
     loop twin.
   - `ops.md` — how ops work (PyTorch fidelity, broadcasting, validation in
     Rust, errors that name shapes), `torch ops`, `--help` per op; links to the
     generated reference (placeholder link until Exp 3 fills it).
   - `install-from-source.md` — clone, bootstrap.sh, install.sh, the relocatable
     layout, `--version`.
5. **Header Docs link goes live**: `/docs` replaces the GitHub placeholder from
   Experiment 1.
6. **Honest content only**: every command shown must be one the current binaries
   actually accept; outputs shown are real (taken from the README, the issue
   records, or run live during writing). No speculative features.
7. **Staleness guard for the install block**: the brew three-command block
   appears on the landing page and in getting-started; they must stay identical
   — a tiny `src/lib/install.ts` exports the canonical string the landing page
   imports, and a concrete checker — `website/scripts/check-content.ts` (bun) —
   extracts the first ```bash fence from `getting-started.md` and diffs it
   byte-for-byte against the export, exiting non-zero on drift. (The landing
   page refactors to import the string; no visual change.)
8. **The docs markdown is dprint-formatted like all repo markdown** (review
   catch: `dprint.json` includes `**/*.md` with no excludes, so the content was
   never exemptable by prose) — `dprint fmt` runs on
   `website/src/content/docs/**/*.md` and the check gates on it. dprint leaves
   non-formattable fences (bash/nu) untouched — verified empirically in review —
   so this composes with the install-block guard; `json` fences ARE reformatted
   by the json plugin, so json outputs are shown dprint-formatted, not
   byte-faithful to raw tool output (semantics unchanged).

## Changes

1. **`website/src/content.config.ts`** (NEW): the typed `docs` collection.
2. **`website/src/content/docs/*.md`** (NEW): the 8 pages above.
3. **`website/src/pages/docs/[...slug].astro`** + **`/docs` index route** (NEW):
   collection routing.
4. **`website/src/components/DocPage.astro`** (NEW): sidebar + article +
   prev/next; **`global.css`** gains `.prose-nutorch`.
5. **`website/src/components/Header.astro`**: Docs → `/docs`.
6. **`website/src/lib/install.ts`** (NEW) + landing-page refactor to use it.
7. **`website/scripts/check-content.ts`** (NEW): install-block byte equality +
   op-name membership against `torch ops --json`.
8. **No Rust changes; `v1/` untouched.**

## Verification

1. **Build gate**: `bun install --frozen-lockfile && bun run build` exits 0 (the
   known upstream DEP0205 aside); all `/docs/*` routes emitted to `dist/`.
2. **Collection-driven sidebar**: every one of the 8 pages appears in the built
   sidebar in `order` order with sections; the current page is highlighted;
   prev/next links chain correctly end to end.
3. **Markdown Shiki path proven** (the half not exercised by Experiment 1):
   built docs pages contain dual-theme `.astro-code` blocks with `--shiki-dark`
   spans for `bash`, `nu`, and `json` fences — and the glue applies (same class,
   already-proven rule).
4. **Content honesty spot-check**: every `torch`/`nutorch` invocation shown in
   the docs is validated against the real binaries. Mechanical half:
   `check-content.ts` also extracts every `torch <op>` op name used in fences
   and asserts membership in `torch ops --json`. Enumerated non-op surface,
   checked live against the brew-installed binary:
   `torch daemon
   status|ttl|stop|restart`, `torch tensors [--json]`,
   `torch free [--all]`, `torch value [--meta]`,
   `torch backward|grad|zero_grad|detach`, `torch nn ...`/`forward`,
   `torch ops [--json]`, `torch nu-module`, `torch --version`, `NUTORCHD_TTL`.
   Mismatches are findings.
5. **Both modes, by screenshot**: getting-started and neural-networks pages
   captured light and dark via the Experiment-1 harness; sidebar, prose, and
   code blocks all legible and on-brand in both.
6. **Hygiene**: dprint clean on ALL touched md/json INCLUDING the new docs
   markdown (decision 8); `check-content.ts` green; `v1/` untouched; Rust suite
   untouched.

**Pass** = all six. **Fail** = routes missing, sidebar hand-maintained after
all, flat code blocks on docs pages, or any documented command the binaries
reject.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only,
verifying against the installed Astro 6.4.6 and live dprint behavior). **First
pass: CHANGES REQUIRED** — 1 Required: the design tried to exempt the docs
markdown from dprint, but `dprint.json` includes `**/*.md` with no excludes —
the content was never exemptable, and hand-formatted prose would fail the repo's
own gates. Absorbed as decision 8: the docs markdown is dprint-formatted like
all repo markdown. Optionals folded: `/docs` pinned to render Getting Started
directly (no static meta-refresh hop); `check-content.ts` specified as the
concrete install-block byte-diff + op-membership checker; the non-op verb
surface enumerated for the honesty gate. Nit folded: `render(entry)` from
`astro:content` pinned (Astro 6 removed `entry.render()`). The reviewer
confirmed the load-bearing reuse claims: `src/content.config.ts` + glob loader
match the installed v6 API, and markdown fences emit `.astro-code` covered by
Experiment 1's glue. **Second pass: APPROVED** — all five fixes verified in
place; the reviewer empirically tested dprint on scratch fences and found
bash/nu fences byte-preserved but `json` fences reformatted by the json plugin.
Folded: decision 8 narrowed accordingly (json outputs shown dprint-formatted,
not byte-faithful), and the `/docs` duplicate-URL pair gets a canonical link to
the slug URL.

## Result

**Result:** Pass

The docs exist: 8 written pages, a collection-driven sidebar, and the markdown
Shiki path proven — 10 routes built.

- **System**: `src/content.config.ts` (glob loader + zod schema:
  title/description/order/section), `[...slug].astro` over the collection via
  `render(entry)`, `/docs` rendering Getting Started directly with
  `<link rel="canonical">` to the slug URL (asserted in built HTML).
- **The pages** (all dprint-formatted per decision 8): getting-started, daemon,
  tensors, ops, autograd, neural-networks, nushell, install-from-source —
  sections Start / Core / Deep learning / Clients / Install, in `order` order
  (asserted: sidebar link sequence, section labels, `aria-current` + brand-green
  highlight on the current page, prev/next chain checked on a middle page).
- **One real bug caught by the screenshot gate**: the single-`<details>` sidebar
  was INVISIBLE on desktop — a closed `<details>` hides its children regardless
  of CSS, so `md:block` could not resurrect it. Fixed by rendering the nav twice
  through a shared `DocNav.astro`: a `md:hidden` disclosure for mobile and a
  `hidden md:block` block for desktop. (The design's "details + md:block" sketch
  was wrong; the reviewer's approval of it and my implementation both missed it
  until the screenshots.)
- **Markdown Shiki proven**: docs pages emit `.astro-code` blocks with
  dual-theme spans (nushell page: 5 blocks, 185 `--shiki-dark` spans) from
  `markdown.shikiConfig` — the Experiment-1 glue covers them, and the light/dark
  screenshots show code visibly switching.
- **Content honesty** (with one humbling result-review catch, below):
  `check-content.ts` green — install block byte-identical to `install.ts`
  (survived dprint, as predicted for bash fences), every `torch <verb>` in every
  fence either in `torch ops --json` (185 names) or in the verified non-op set.
  The checker CAUGHT a real error during writing: my drafted training loop used
  `torch nn step`; the real verb is top-level `torch step` (and `torch nn sgd`
  takes the MODULE handle, not parameters — fixed from the committed acceptance
  script). Every documented non-op verb run live against the brew binary: daemon
  start/status/ttl/restart/stop (+ --json), tensors (+ --json), free (+ --all),
  value --meta, backward/grad/zero_grad/detach, nn
  linear/sgd/parameters/info/zero_grad, forward, step, ops --json, nu-module,
  --version — all OK.
- **Screenshots**
  (`logs/issue-0012/docs-{getting-started,neural-networks}-{light,dark}.png`):
  sidebar, prose, inline code chips, and fenced blocks legible and on-brand in
  both modes.
- **Gates**: build 0 errors (10 pages; the known upstream DEP0205 only);
  frozen-lockfile green; dprint clean on the docs markdown + package.json; Rust
  tree untouched; `v1/` untouched.

## Conclusion

The documentation system is real and the eight core pages are live. Two lessons
recorded: screenshot gates catch what HTML assertions cannot (the
details-element sidebar bug), and the honesty checker pays for itself
immediately (the `nn step` fiction never reached a commit). Experiment 3
generates the ops reference from the table into this same collection — the
sidebar will pick it up by construction. Experiment 4 finishes search, sitemap,
and OG polish.

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context), reviewing BEFORE
the result commit. **First pass: CHANGES REQUIRED** — the reviewer found the
experiment's own Fail condition triggered: `tensors.md` documented
`torch arange 0 10 2`, which the real binary REJECTS (the true form is
`torch arange 10 --start 0 --step 2`). The honesty checker could not catch it —
it validates verb NAMES against `torch ops --json`, not argument grammar — and
the live spot-checks had covered the non-op surface only. Both Required findings
fixed: the line corrected and verified live (`[0.0,2.0,4.0,6.0,8.0]`), and the
checker's name-only scope is now recorded here as a known limitation (op
argument forms are verified by live execution, which the reviewer extended
across the training loop, autograd verbs, all 19 module kinds, save/load, the
dual-input pipe form, the `value --meta` and non-finite round-trips, and the
185-op count — all confirmed honest). Nit folded: the nushell-page Shiki figures
corrected to 5 blocks / 185 spans (the "10" had counted doubled class-string
occurrences). The reviewer also adversarially tested the checker itself —
corrupted copies fail on both install-block drift and unknown verbs, so the gate
can actually fail — and confirmed the details-element bug narrative, the
canonical link, both navs, the prev/next chain ends, the screenshots' genuine
mode differences, and the process state (result uncommitted, plan commit
`ab642d8` plan-only). **Second pass: APPROVED** — the corrected `arange` line
re-verified live by the reviewer; no remaining findings.
