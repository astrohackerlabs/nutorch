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

# Experiment 1: Typed handles and the typed registry

## Description

The foundation everything else in this issue stands on: handles become
`tensor://<uuid>` (with `nn://` and `optim://` recognized from day one), and the
registry becomes kind-aware. No modules yet — this experiment is deliberately
the mechanical migration plus the kind machinery, so the review gates can focus
on the one failure mode that matters: a site still minting or accepting bare
UUIDs.

**Decisions, made here (within the issue's recorded contract):**

1. **Parsing lives in the registry.**
   `Handle::parse(&str) ->
   Result<(Kind, Uuid…), …>` is a registry-module
   concern; dispatch and the client stay string-agnostic (they already are —
   handles flow through both untouched). The registry keys by bare UUID
   internally and mints prefixed strings.
2. **Kinds recognized now: `tensor`, `nn`, `optim`.** Only tensor entries EXIST
   yet, but the parser accepts all three prefixes so the error space is complete
   from the start:
   - bare/malformed handle → `bad_argument`:
     `malformed handle: expected tensor://<id>, nn://<id>, or optim://<id>` (the
     clean break — bare UUIDs are NOT grandfathered, per the issue);
   - unknown prefix → the same malformed error, naming the valid kinds;
   - right prefix, absent id → `unknown_handle` (as today);
   - wrong prefix on a REAL object → new error code **`wrong_kind`**:
     `handle refers to a tensor, not a module` — the error the typed scheme
     exists to make possible.
3. **Typed accessors.** `registry.get_tensor(handle)` (and later
   `get_module`/`get_optimizer`) returns `Result<&Tensor, Lookup>` where
   `Lookup` distinguishes Unknown/WrongKind/Malformed; dispatch maps those to
   the three codes above. `insert` becomes `insert_tensor` (minting
   `tensor://…`). `free`, `touch`, `contains`, `list` parse the same way —
   `free` works on any kind by construction (today: tensors).
4. **`torch tensors` prints full handles** (`tensor://…` in column 1) — the
   awk→free composition keeps working because `free` accepts exactly what the
   listing prints.
5. **The Entry enum lands with ONE variant** (`Entry::Tensor`). Module and
   Optimizer variants arrive with their own experiments — no speculative dead
   code; the enum existing is what later experiments extend.

**Scope of the sweep**: every internal mint/lookup site (registry, the golden
harness's `T<i>` convention, unit-test helpers), a sentence in the README
workflow prose introducing the handle scheme (no literal handle exists there
today — verified), and the `unknown handle` message format. Client and protocol:
zero changes (strings in, strings out — verified, not assumed).

## Changes

1. **`nutorchd/src/registry.rs`**: `HandleKind` (`Tensor`/`Module`/ `Optimizer`,
   with prefix strings `tensor`/`nn`/`optim`); handle parse/mint helpers;
   `Entry` enum (one variant); `Lookup` error enum; `insert_tensor`,
   `get_tensor`, kind-agnostic `free`/`touch`/
   `contains`/`len`/`approx_bytes`/`list` (listing carries kind, shown by
   `torch tensors` only for tensors today). Unit tests: mint/parse round-trip;
   all four error shapes (malformed, unknown prefix, absent id, wrong kind — the
   last via a hand-built `nn://` reference to a real tensor's uuid).
2. **`nutorchd/src/dispatch.rs`**: lookup sites move to the typed accessors; the
   `Lookup` → error-code mapping (`unknown_handle`, `wrong_kind`,
   `bad_argument`); existing tests updated ONLY where they assert handle formats
   or construct fake handles. **The observable semantic change has exactly TWO
   reachable paths, both flipping `unknown_handle` → `bad_argument` for bare
   strings**: (a) operand/`value`/`free` lookups (the `"nope"` tests — including
   `free_is_atomic_…`, whose validate-before-remove invariant is re-verified
   under a MALFORMED middle handle; `double_free_errors_visibly` stays
   `unknown_handle` since its handle is well-formed-but-absent), and (b) the
   `HandleOrScalar` typo path from issue 0005 exp 5
   (`torch pow $b
   notahandle`) — the torch-cli comment documenting that UX as
   `unknown_handle` is updated in the same sweep (comment-only; the client's
   code is untouched, so the no-client-changes Fail criterion is judged against
   code, not comments).
3. **`nutorchd/tests/golden.rs`**: agnostic already (handles flow from `insert`
   outputs); verified, plus the `T<i>` substitution confirmed prefix-clean.
4. **`README.md`**: one sentence in the workflow section introducing the handle
   scheme (`tensor://…` now; `nn://`/`optim://` arriving with this issue).
5. **No client, protocol, or ops changes** (the Fail criterion if violated).

## Verification

1. **Hygiene**: build 0 warnings; fmt/dprint clean on touched files; full suite
   green (the 220 goldens REGENERATE byte-identically — handles never appear in
   golden.json, verified).
2. **Unit tests**: the registry error-shape quartet; dispatch mapping.
3. **Live**:
   - `torch tensor '[1,2]'` prints `tensor://<uuid>`; the full PoC pipeline
     works end to end with prefixed handles;
   - `torch tensors | awk '{print $1}' | torch free` still empties the registry
     (the composition over prefixed handles);
   - `torch value <bare-uuid>` → malformed-handle error naming the expected
     forms, exit 1;
   - `torch value nn://<real-tensor-uuid>` → `wrong_kind`, message naming both
     kinds, exit 1;
   - `torch value tensor://<absent-uuid>` → `unknown handle`, exit 1;
   - autograd flow unchanged: `randn --requires_grad` → backward → grad with
     prefixed handles throughout.
4. **The sweep is complete**: `rg` proves no site mints a bare UUID (every
   `insert` path goes through the minting helper) and no test asserts a
   bare-UUID format.

**Pass** = all four. **Fail** = client/protocol/ops changes were needed, or any
handle escapes unprefixed.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**Verdict: APPROVED — no Required findings (first pass).** Two Optional and one
Nit, all folded in: the observable semantic change has TWO reachable paths, not
one — the design now names the `HandleOrScalar` typo path (whose torch-cli
comment is updated, comment-only) alongside the operand path; the atomicity free
test is listed explicitly with its invariant re-verified under a malformed
middle handle (and `double_free_errors_visibly` correctly stays
`unknown_handle`); and the README claim was corrected (no literal handle exists
there — prose sentence instead). The reviewer independently verified the
load-bearing claims: the call-site sweep is COMPLETE (every `registry.` call
traced; all in dispatch.rs and golden.rs; client/protocol/lifecycle/ops have
zero handle logic), golden.json contains zero handle-like strings (the
byte-identical regeneration claim is sound), the client is genuinely
string-agnostic, `wrong_kind` reachability is correctly scoped, the
single-variant Entry enum produces no dead-code warnings (all HandleKind
variants are constructed in parse), and no code anywhere assumes a bare-UUID
format.

## Result

**Result:** Pass

Every handle in nutorch is now typed. The migration landed exactly inside its
declared boundary: registry rewritten (HandleKind, Object/Entry enums, Lookup
errors, typed accessors), dispatch call sites moved to the typed API, test code
updated, README sentence added — zero client, protocol, or ops code changes (the
torch-cli edit is the one comment the design declared).

- **Live, the error quartet**: bare UUID →
  `malformed handle: … (expected
  tensor://<id>, nn://<id>, or optim://<id>)`;
  `nn://<real-tensor-id>` → `handle … refers to a tensor, not a module`
  (`wrong_kind` — the error the scheme exists for); `tensor://absent` →
  `unknown handle`; `blob://…` → malformed. All exit 1.
- **The composition holds**: `torch tensors | awk '{print $1}' | torch
  free`
  empties the registry over prefixed handles; the PoC pipeline and the full
  autograd flow run end to end with `tensor://…` throughout.
- **The second observable path confirmed**: the `HandleOrScalar` typo
  (`torch pow $c notahandle`) now reports malformed-handle, and the torch-cli
  comment documenting the old UX was updated (comment-only).
- **Suite**: 59 unit tests (4 new registry error-shape tests; the two predicted
  flips landed — `free_is_atomic…` holds its validate-before-remove invariant
  under a MALFORMED middle handle; `double_free` correctly stays
  `unknown_handle`), 220 goldens green and golden.json **byte-identical**
  (handles never appear in it, as verified), 0 warnings, fmt/dprint clean, `v1/`
  untouched.
- **Sweep completeness**: `rg` proves UUID minting exists only inside the
  registry, and no legacy `registry.insert/get/contains` call survives.
- **One refinement found by the new tests**: the WrongKind error originally
  blamed the ACCESSOR's expectation, producing "refers to a tensor, not a
  tensor" for a wrong prefix — the message now blames the PREFIX's claim (what
  the user wrote) and falls back to the accessor's expectation only when the
  prefix is truthful.
- **One process note**: two background-task writes to dispatch.rs were lost
  mid-implementation (a file-state race between overlapping background builds
  and edits); re-applied in the foreground with immediate `rg` verification.
  Lesson recorded: edits to a file and builds touching it stay sequential.

## Conclusion

The foundation holds: kinds parse, mint, and kind-check exactly per the issue's
recorded contract, with the error space complete before a single module exists.
Experiment 2 can now add `Object::Module` as one more enum variant plus its
accessors — the shape this experiment exists to make cheap. Next: the module
foundation (`nn linear`, `relu`, `sequential`, `forward`, `parameters`) and the
VarStore-vs-own-parameters decision.

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only),
reviewing the pre-commit working tree. **Verdict: APPROVED — no Required,
Optional, or Nit findings.** The reviewer verified the issue's decision-3
contract directly against the code (bare-UUID keys, prefixed minting,
three-prefix parsing), traced the WrongKind attribution in BOTH directions —
including the future `get_module` fallback branch, judged correctly written
though unreachable until Experiment 2 — reproduced the live error quartet
verbatim, read both flipped tests (atomicity with both survivors asserted under
a malformed middle; double_free correctly retaining `unknown_handle` for
well-formed-but-absent), confirmed sweep completeness by rg (UUID minting only
in the registry; zero legacy calls; torch-cli diff comment-only), proved
golden.json handle-free and byte-unmodified, and re-ran the end-to-end flows
including autograd. The process note about the lost background writes was
checked and judged candid and immaterial to the final state.
