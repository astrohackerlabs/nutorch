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

# Experiment 1: `--json`, the generator, and the module

## Description

The whole issue in one cohesive experiment: the `--json` output mode, the
`torch nu-module` generator, the hand-written prelude, the committed
`nutorch.nu`, and verification in real Nushell (0.113.0, on this machine at
`/opt/homebrew/bin/nu` — design question 4 answered).

**Decisions, made here:**

1. **`--json` semantics per verb** (client-side rendering; wire untouched):
   - `torch tensors --json` → the wire's JSON array, raw;
   - `torch daemon status --json` → the wire's JSON object, raw;
   - `torch ops --json` → a JSON array built from the OpSpec table
     (`{name, category, summary}` — the table is client-resident, there is no
     wire form);
   - `torch nn info <m> --json` → a JSON OBJECT: the daemon's `"key: value"`
     lines split on the first `": "` into record fields (values stay strings —
     `"2 tensor(s), 9 element(s)"` is a display string, not data; recorded as
     such).
2. **Wrapper shape — pipeline-first**: for `Exactly(n)` ops the FIRST tensor
   slot comes from `$in` (piped handle), remaining slots are positional;
   creation ops (`Exactly(0)`) take no input. **Variadic ops (design-review
   finding)**: a nu LIST piped to an external renders as box-drawing TABLE
   glyphs (probed — `["a"] | ^cat` emits `╭───┬…`), so "literally pipe `$in`"
   corrupts variadic input; variadic wrappers accept a list `$in` and explicitly
   `str join (char newline)` before piping (one line of conversion logic, named
   honestly), plus `...rest` positionals. For scalar `$in` (the common case) the
   wrapper does pipe it raw — nu sends strings without a trailing newline, which
   the CLI's `lines()` reader handles. **Recorded trade**: the wrappers are
   pipeline-first by design — the bare all-positional form (`torch add $a $b`)
   is intentionally not exposed in Nushell (the CLI retains it); an unpiped call
   errors cleanly ("expected 1 piped handle(s), got 0" — probed).
3. **ParamKind → Nushell signature mapping**: Int → `int`, Float/Scalar →
   `number`, IntList → `list<int>` (serialized to the CLI's JSON form), Bool →
   `--flag` switch, Str → `string`, HandleOrScalar → `any` (numbers pass as
   numbers, strings as handles — the CLI's own rule).
4. **Returns**: `Handles(1)` → the handle string (trimmed);
   `Handles(n)`/`VariableHandles` → `list<string>` (split lines); `Value` →
   `from json` plus the NaN mapping; `None` → nothing.
5. **NaN mapping is REAL floats** (probed: nu 0.113's `'NaN' | into
   float`
   yields NaN): `nutorch value` post-processes its decoded data, recursively
   replacing the dialect tokens (`"NaN"`/`"Infinity"`/`"-Infinity"`) with float
   NaN/±infinity. `nutorch tensor` does the REVERSE on input (nu's `to json`
   would otherwise emit `null` for non-finite floats — probed), so round trips
   are lossless in Nushell without the user touching the dialect.
6. **The committed module lives at the repo root** (`nutorch.nu`, so
   `use nutorch.nu *` works from a checkout). Staleness guard: a `#[test]` in
   torch-cli regenerates the module text and asserts it equals
   `include_str!("../../nutorch.nu")` — the golden byte-stability pattern
   reapplied to a generated artifact.
7. **The generator is `torch nu-module`** (prints to stdout; the committed file
   is its redirected output). The prelude (bespoke verbs: `tensor`, `value`,
   `free`, `tensors`, `ops`, `nn` family, `forward`, `step`, `daemon`
   passthrough) is a static string in the generator source; table-op wrappers
   are emitted from `nutorch_ops::OPS` — the fourth consumer of the single
   source of truth.
8. **A committed Nushell demo**: `scripts/train-regression.nu` — the zsh
   acceptance script's twin, asserting the same thresholds natively (private
   socket, seeded), proving the training loop reads naturally in Nushell.

## Changes

1. **`torch-cli/src/main.rs`**: the `--json` flag on the four structured verbs —
   **which reach the client through four DIFFERENT paths, each needing its own
   plumbing (design-review finding)**: the `ops` early branch returns before
   `parse_raw` and must learn to read its argv; `run_tensors` currently rejects
   ALL flags and must accept this one; `daemon status` parses with `spec = None`
   where `--json` would be treated as value-taking (it joins the bespoke
   presence-flag set); `nn info` branches inside the nn dispatch. Plus the
   `nu-module` verb + `generate_nu_module()` (prelude string + OpSpec-driven
   emission) and the staleness `#[test]`.
2. **`nutorch.nu`** (committed, generated): the prelude + one
   `def "nutorch <op>"` per table op (172 ops) with faithful signatures.
3. **`scripts/train-regression.nu`** (committed): the Nushell twin.
4. **`README.md`**: a Nushell section (use, the three-line example, the pointer
   to regeneration).
5. **No daemon, protocol, or ops changes** (the mandate boundary).

## Verification

1. **Hygiene**: standard; goldens untouched; the staleness test green
   (regenerate-and-diff).
2. **`--json` live**: each of the four verbs round-trips through nu's
   `from json` into structured data.
3. **Nushell sessions live** (real `nu -c`, private sockets):
   - round trip: `[[1 2] [3 4]] | nutorch tensor | nutorch value` returns the
     native table, exactly;
   - non-finite, BOTH directions: a div-by-zero tensor's `nutorch value`
     contains REAL NaN/infinity floats; AND a native nu list containing
     NaN/Infinity (via `'NaN' | into float`) pipes INTO `nutorch tensor` and
     reads back losslessly — without the input-side recursive token substitution
     (NaN detectable only via `$x != $x` — probed), nu's `to json` silently
     emits `null` for non-finite floats;
   - the census composition:
     `nutorch tensors | where bytes > 100 | get handle | each {|h| nutorch free $h }`;
   - autograd: the issue-0008 workflow in nu syntax, grad = 2x;
   - `nutorch ops | where category == loss` returns nine rows;
   - `scripts/train-regression.nu` PASSES with the zsh script's thresholds.
4. **The zsh scripts still pass** (regression guard — the `--json` flag must not
   disturb default rendering).

**Pass** = all four. **Fail** = daemon/protocol/ops changes needed, or wrapper
signatures could not faithfully carry a ParamKind.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only, probing
real nu 0.113). **First pass: CHANGES REQUIRED** — 2 Required: (1) the variadic
"literally pipe `$in`" claim was a corruption trap — nu renders a piped LIST as
box-drawing table glyphs (probed with hexdump), which the CLI would ingest as
garbage handles; variadic wrappers now `str join (char newline)` explicitly, and
the no-argument-logic claim was withdrawn. (2) The `--json` flag reaches the
client through four DIFFERENT code paths, three of which reject or mis-parse
flags today (`ops` never sees argv; `tensors` rejects all flags; `daemon`'s
spec-less parse would swallow the next token) — the per-verb plumbing is now
named in Changes. Optional folded: the input-side NaN mapping gains an explicit
both-directions verification (nu's `to json` emits `null` for non-finite —
reprobed — and NaN is detectable only via `$x != $x`). Nit folded: the
pipeline-first trade (no bare all-positional form in Nushell) is recorded. The
reviewer probed and confirmed sound: the include_str path, compact IntList
`to json -r`, scalar `$in` piping without trailing newline, the unpiped-`$in`
clean error, float precision through stringification, the nn-info colon split,
and nu's assert facilities.
