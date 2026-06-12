+++
[implementer]
agent = "claude-code"
model = "claude-fable-5"

[review]
waived = "user decision 2026-06-12: no adversarial review for issue 0013"
+++

# Experiment 4: The three-state theme — system / light / dark

## Description

Punch-list addition: the theme control grows a third state. Today the toggle is
binary — and one-way: the first click writes `light` or `dark` to localStorage,
and from then on the OS preference is dead to the site. There is no way back to
"follow my system." The fix is the standard three-state model: **System**
(follow the OS, live), **Light** (pinned), **Dark** (pinned).

**Decisions, made here:**

1. **Two-layer state, so the CSS never changes.** The SETTING
   (`system | light | dark`) lives in localStorage (`theme` key; `system` stored
   explicitly, and treated as the default when the key is absent or unknown).
   The RESOLVED mode (`light | dark`) is what lands on `:root[data-theme]` —
   exactly the attribute every existing rule keys on (tokens, Shiki glue, toggle
   icons, Astrohacker logos). Zero churn in the token sheet; the resolver is the
   only new logic.
2. **System mode is LIVE**: in `system`, a
   `matchMedia("(prefers-color-scheme: dark)")` change listener re-resolves
   `data-theme` on the spot — change the OS appearance and the open page
   follows, no reload. In `light`/`dark` the listener is ignored.
3. **The control stays one button, now cycling** system → light → dark → system.
   It shows the CURRENT setting's icon — monitor (system), sun (light), moon
   (dark) — switching from the old show-the-target convention to show-the-state,
   which is the only legible choice once there are three states.
   `aria-label`/`title` name the current setting and the next one ("Theme:
   system — click for light"). The setting also lands on
   `:root[data-theme-setting]` so the icon swap stays pure CSS, same pattern as
   today's sun/moon rules.
4. **The no-flash init script grows the resolver but keeps its contract**: still
   inline in `<head>`, still respects a pre-set `data-theme` attribute first
   (the screenshot harness and any future SSR depend on that escape hatch), then
   reads the setting and resolves. The toggle button's inline handler updates
   BOTH attributes and persists the setting.
5. **Verified mechanically via CDP, not just by eye**: the issue-0012 search
   harness proved headless Chrome + CDP is cheap; a small
   `scripts/check-theme.ts` drives a served page through the full matrix — cycle
   order and persistence across reload (localStorage), pinned modes ignoring an
   EMULATED OS flip (`Emulation.setEmulatedMedia`), and system mode following
   the same flip live without reload. That last assertion is the one screenshots
   cannot make.
6. **Docs/site untouched otherwise** — this is `Base.astro` (init script),
   `ThemeToggle.astro` (markup + handler), `global.css` (icon rules), and the
   new check script. No content changes.

## Changes

1. **`website/src/layouts/Base.astro`**: the init script resolver (setting →
   resolved mode → both attributes; live matchMedia listener for system mode).
2. **`website/src/components/ThemeToggle.astro`**: three icons, cycle handler,
   `aria-label`/`title` per state.
3. **`website/src/styles/global.css`**: icon-visibility rules keyed on
   `data-theme-setting`.
4. **`website/scripts/check-theme.ts`** (NEW): the CDP matrix gate.
5. **Nothing else** — no content, no Rust, no `v1/`.

## Verification

1. **Build + existing gates**: `bun run build` clean; `check:content`,
   `check:links`, `check:ops-ref` green.
2. **The CDP matrix** (`check-theme.ts` against a served build):
   - fresh visit (no localStorage) → setting `system`, resolved mode equals the
     emulated OS scheme;
   - OS flip WHILE in system mode → `data-theme` follows live, no reload;
   - click → `light` (pinned): OS flip ignored; reload keeps `light`;
   - click → `dark` (pinned): same checks;
   - click → back to `system`: resolved mode tracks the emulated OS again;
   - localStorage holds the SETTING (`system`/`light`/`dark`), and `data-theme`
     only ever holds a RESOLVED mode.
3. **Both visual modes intact**: the exp-1 screenshot harness still works
   (pre-set `data-theme` respected); landing-page light/dark screenshots
   unchanged in character — tokens, code blocks, and the Astrohacker logo all
   still switch.
4. **Accessibility floor**: the button exposes a state-naming `aria-label`;
   icons are `aria-hidden`.
5. **Hygiene**: dprint clean on touched files; zero `.rs` diffs; `v1/`
   untouched.

**Pass** = all five, with every CDP matrix row green. **Fail** = any pinned mode
that follows the OS, a system mode that needs a reload, or a localStorage value
that leaks a resolved mode instead of the setting.

## Result

**Result:** Pass

System / light / dark, with the system state proven live by an emulated OS.

- **The two-layer model landed as designed**: localStorage holds the SETTING
  (`system` stored explicitly), `data-theme` only ever holds a RESOLVED mode,
  and `data-theme-setting` drives the icon — the token sheet, Shiki glue, and
  Astrohacker logo rules needed zero changes.
- **The CDP matrix is all green** (`check:theme`, 14 assertions): fresh visit
  defaults to system with empty storage; an emulated OS flip moves the page LIVE
  in system mode (no reload); pinned light and dark each ignore OS flips and
  persist across reloads; the third click returns to system, stores it
  explicitly, and tracking resumes live; `data-theme` never leaks a setting; the
  button's `aria-label` names the state and the next action ("Theme: system —
  click for light").
- **The control**: one cycling button, monitor/sun/moon showing the CURRENT
  setting, icons `aria-hidden`, label updated per click.
- **The harness contract held**: pre-set `data-theme` pages still short-circuit
  the init script; the re-taken landing screenshots render both modes distinctly
  (the pre-resolved pages show the system icon — the designed default when no
  setting attribute exists).
- **Two implementation notes**: Chrome's DevTools endpoint needs an isolated
  `--user-data-dir` (a bare launch can delegate to a running Chrome and exit)
  and a startup POLL rather than a fixed sleep — both now baked into
  `check-theme.ts`.
- **Gates**: build clean (20 pages); `check:content`/`check:links`/
  `check:ops-ref`/`check:theme` green; dprint clean; zero `.rs` diffs; `v1/`
  untouched.

## Conclusion

The theme control now does what every well-behaved site control does: defaults
to the OS, follows it live, and pins only when asked — with the behavior locked
in by an executable gate (`check:theme`) rather than a promise. The CDP harness
pattern has now paid for itself twice; it is the house tool for anything
screenshots can't assert.
