+++
[implementer]
agent = "claude-code"
model = "claude-fable-5"

[review]
waived = "user decision 2026-06-12: no adversarial review for issue 0013"
+++

# Experiment 2: Apache → MIT

## Description

Punch-list fix 3: the project license changes from Apache-2.0 to MIT. The full
inventory of license statements was taken before designing (grep across the repo
and the published tap); this experiment flips every live one and nothing else.

**The inventory (verified 2026-06-12):**

| Where                                 | Today                                     |
| ------------------------------------- | ----------------------------------------- |
| `LICENSE` (repo root)                 | Apache 2.0 full text                      |
| `nutorchd/Cargo.toml` line 5          | `license = "Apache-2.0"`                  |
| `ops/Cargo.toml` line 5               | `license = "Apache-2.0"`                  |
| `torch-cli/Cargo.toml` line 5         | `license = "Apache-2.0"`                  |
| `dist/nutorch.rb` line 11             | `license "Apache-2.0"`                    |
| Published tap `Formula/nutorch.rb` 11 | `license "Apache-2.0"` (OUTWARD)          |
| `website/src/components/Footer.astro` | two `Apache-2.0` strings (exp 1 hooks)    |
| `README.md` `## Copyright`            | `Copyright (C) 2025-2026 Identellica LLC` |

Nowhere else: the workspace `Cargo.toml` carries no license key, the docs
content and tap README never name the license, there is no `NOTICE` file, and
`v1/` has no license file of its own (the frozen archive stays frozen — history
records that the project WAS Apache through v0.1.0).

**Decisions, made here:**

1. **The MIT text follows house style**: shannon and termsurf both ship
   byte-similar `LICENSE` files reading `MIT License` /
   `Copyright (c) 2026 Astrohacker`. nutorch's new `LICENSE` is the same
   standard MIT text with the same holder — consistent with fix 1's "copyright
   Astrohacker" direction.
2. **The README's `## Copyright` section changes to match the LICENSE**:
   `Copyright (C) 2025-2026 Identellica LLC` becomes
   `Copyright (c) 2026 Astrohacker` with the MIT sentence and a link to
   astrohacker.com. (Flagged explicitly because Identellica LLC is a THIRD name
   found during inventory — if the user wants Identellica or a year range
   retained in either file, this is the decision to amend.)
3. **Cargo manifests**: the three member crates flip to `license = "MIT"`.
4. **The formula flips in both places**: `dist/nutorch.rb` (source of truth) and
   the published tap's `Formula/nutorch.rb`. The tap edit is this experiment's
   ONE outward-facing action: a single commit to `nutorch/homebrew-nutorch`
   changing the license line, pushed. brew treats the license field as metadata
   — no version bump, no re-bottle (the bottled keg's embedded LICENSE stays
   Apache until the next tagged release naturally carries the MIT file; recorded
   in the issue spine as accepted).
5. **The footer hooks flip**: both `Apache-2.0` strings in `Footer.astro` become
   `MIT` (the donor sites' exact wording is "MIT License." in the copyright
   line; ours renders `© 2026 Astrohacker · MIT License`, and row 1
   `nutorch · MIT`).
6. **Vendored-component attribution is unaffected and stays as-is**: libtorch
   (PyTorch, BSD-3) ships in bottles with its own license terms regardless of
   what nutorch's license is; Apache→MIT changes nothing about that obligation
   either way (recorded, not expanded here).
7. **No retroactive re-release**: v0.1.0's tag, source tarball, and bottle
   remain Apache-licensed artifacts; the next release is MIT end to end.

## Changes

1. **`LICENSE`**: Apache 2.0 text → house MIT text.
2. **`nutorchd/Cargo.toml`, `ops/Cargo.toml`, `torch-cli/Cargo.toml`**:
   `license = "MIT"`.
3. **`dist/nutorch.rb`**: `license "MIT"`.
4. **`website/src/components/Footer.astro`**: both strings → MIT wording.
5. **`README.md`**: `## Copyright` section → Astrohacker + MIT + link.
6. **Tap repo `Formula/nutorch.rb`** (outward): `license "MIT"`, one commit,
   pushed.
7. **No other files; `v1/` untouched.**

## Verification

1. **Zero live Apache references**:
   `grep -ri "apache" --exclude-dir=v1
   --exclude-dir=node_modules --exclude-dir=dist --exclude-dir=.venv-torch
   --exclude-dir=target --exclude-dir=docs .`
   over the repo returns ONLY historical records (issue files for
   0011/0012/0013, which are immutable or are this experiment's own text) — no
   LICENSE, manifest, formula, website, or README hits.
2. **Cargo accepts the manifests**: `cargo build --release` clean (the license
   key is metadata, but the build proves the TOML parses); the Rust suite
   untouched and green.
3. **Website**: `bun run build` clean; built footer shows
   `© 2026 Astrohacker · MIT License` and `nutorch · MIT`; both-mode screenshots
   of the footer re-taken; all site gates green.
4. **The tap**: `brew audit` (or at minimum `brew style`/a formula parse)
   accepts `license "MIT"` locally before the push; after the push, the raw
   GitHub formula shows the MIT line; `brew install` NOT re-run (no functional
   change).
5. **Hygiene**: dprint clean on touched md/json; `cargo fmt -- --check` clean
   (no Rust source touched); `v1/` untouched.

**Pass** = all five. **Fail** = any live Apache string survives outside frozen
history, or the tap push changes anything beyond the one line.

## Result

**Result:** Pass

nutorch is MIT, everywhere it lives.

- **`LICENSE`**: now the house MIT text, byte-identical to shannon's
  (`MIT License` / `Copyright (c) 2026 Astrohacker`).
- **Manifests**: all three crates carry `license = "MIT"`;
  `cargo build --release` clean (0 warnings), `cargo fmt -- --check` clean.
- **Formula, both copies**: `dist/nutorch.rb` and the published tap's
  `Formula/nutorch.rb` (tap commit `10d3356`, pushed; the raw GitHub formula
  shows `license "MIT"` at line 11). `brew style` reports only the PRE-EXISTING
  `std_cargo_args` suggestion (our install deliberately uses `cargo build` for
  the `.libtorch` symlink layout — issue 0011) — nothing introduced by this
  change. One operational note: `brew tap` had re-cloned the local tap over
  HTTPS during issue 0011's cold-user test, so the push needed the remote
  switched back to SSH first.
- **Website**: footer renders `nutorch · MIT` and
  `© 2026 Astrohacker · MIT License` (asserted in built HTML + both-mode
  screenshots, `logs/issue-0013/mit-footer-{light,dark}.png`); all site gates
  green (20 pages, check:links/content/ops-ref).
- **README**: `## Copyright` now
  `Copyright (c) 2026 Astrohacker — MIT
  License` with links to astrohacker.com
  and LICENSE.
- **One inventory miss, caught by the gate**: the zero-Apache grep found
  `AGENTS.md`'s directory-structure comment (`LICENSE  # Apache 2.0`) — fixed to
  `# MIT`. The verification grep now returns ZERO live Apache references outside
  `issues/` history and the frozen `v1/`.
- **Untouched, as designed**: v0.1.0's tag/tarball/bottle remain Apache-licensed
  history; no re-release; `v1/` frozen.

## Conclusion

Fix 3 is done — the license is MIT end to end in the live project, with the
v0.1.0 artifacts left as honest Apache history. The grep gate proved its worth
by catching the AGENTS.md reference the manual inventory missed. Remaining on
the punch list: fix 2 (the top-of-page add-tensors example) and whatever the
user names next.
