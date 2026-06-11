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

# Experiment 2: The module foundation — linear, activations, sequential, forward

## Description

`Object::Module` becomes real: construction, composition, forward, and parameter
access — the smallest set that proves the object model end to end. Scope:
`linear`, the parameterless activations (`relu`, `sigmoid`, `tanh`, `gelu`),
`sequential`, `forward`, `nn parameters`, `nn info`.

```bash
l=$(torch nn linear 2 3)                  # nn://… handle
m=$(torch nn sequential $l "$(torch nn relu)")
y=$(torch forward $m $x)                  # or: $x | torch forward $m
torch nn parameters $m                    # tensor:// handles, LIVE views
torch nn info $m
```

**The issue's design question 1, decided here: own parameters, no VarStore.**
Evidence from the tch API (verifiable in source, the review's job to attack):

- **Corrected by design review**: the first draft claimed tch "offers no
  supported store merge" — that is FALSE (`VarStore::merge` exists, public,
  non-deprecated). The true comparison: the VarStore path is _workable_ —
  per-module stores, `merge` at composition (it consumes stores by value, which
  even matches our consume semantics), and `nn::Optimizer` plus
  `VarStore::save`/`load` for free — at the cost of per-module store
  bookkeeping, `nn::Path`-based construction, and a by-value merge dance on
  every composition. Own representation costs a hand-rolled optimizer step (SGD
  is three lines; Adam is bookkeeping — its own experiment regardless) and
  per-parameter serialization, which tch covers without VarStore
  (`Tensor::write_safetensors` / `save_multi` — so save/load is NOT foreclosed;
  recorded for the save/load experiment). The decision STANDS on the true
  ledger: direct composition with no store juggling, full control of parameter
  identity (which the live-views contract needs), against costs we were paying
  anyway.
- Own representation costs little because the module set is enumerable:

  ```rust
  enum NnModule {
      Linear { weight: Tensor, bias: Option<Tensor> },
      Relu, Sigmoid, Tanh, Gelu,
      Sequential { children: Vec<NnModule> },
  }
  ```

  `forward` is a match (linear via tch's functional `f_linear`; activations via
  their tensor ops); `parameters()` walks the tree collecting `&Tensor`s.
  Optimizers (a later experiment) hold `shallow_clone` references — tch tensors
  are internally refcounted, so in-place steps propagate to the module. Nothing
  tch's Module trait provides is lost; everything our composition model needs is
  gained.

**Other decisions, made here:**

1. **Module verbs are BESPOKE, not table ops.** The op table stays tensor-only
   (its invariants — broadcasting pre-checks, touch passes, golden harness —
   assume tensor operands). Wire: `{"op":"nn","kind":"linear","args":{…}}` for
   construction, `{"op":"forward","module":"nn://…","tensor":"tensor://…"}`,
   `{"op":"nn_parameters","module":…}`, `{"op":"nn_info","module":…}`.
2. **Construction grammar**: `torch nn <kind> [args…]` — a subcommand verb
   client-side (the `daemon` pattern). `linear <in> <out>` positionals;
   `--no-bias` (presence-only Bool cannot express `bias=false` through a
   faithful `--bias` — the topk `--smallest` lesson, reapplied);
   **`--weight <tensor://…>` and `--bias-tensor <tensor://…>`** optional
   explicit-weight construction (shape-checked: weight `[out, in]`, bias
   `[out]`) — both the golden strategy's loading mechanism AND the user's
   pretrained-weights path. **Pinned semantics (design-review finding)**:
   explicit weights are DEEP-COPIED into the module (state_dict-load semantics —
   the caller's tensor is never aliased or mutated), and the module's parameters
   get `requires_grad` set LAST on the post-copy tensor regardless of the
   source's setting (PyTorch parameters always track; the issue-0008 non-leaf
   trap honored here too). `--bias-tensor` with `--no-bias` is `bad_argument`;
   `--weight`/`--bias-tensor` on non-`linear` kinds is `bad_argument`. The
   explicit-weight unit test asserts the resulting parameter requires grad AND
   the source tensor's requires_grad is unchanged.
3. **Default init is PyTorch's**: weight and bias both `U(-1/√in, 1/√in)` (what
   `torch.nn.Linear` does), drawn on the seeded CPU generator (the randn
   convention) and moved to MPS, `requires_grad` set LAST (the issue-0008
   non-leaf trap, honored). Bitwise init parity with Python is NOT pledged
   (different draw sequences); determinism daemon-side IS (seeded → identical
   weights).
4. **`sequential` consumes its children — ATOMICALLY.** All child handles are
   validated and resolvable BEFORE any is removed (the `free` atomicity
   invariant, reapplied): a bad middle handle or a duplicate
   (`sequential
   $l $l` — caught by the validate pass seeing the same id
   twice) leaves the registry unchanged. Only after validation do the modules
   move into the composite. Nested sequentials are allowed (a Sequential is a
   module like any other). Empty sequential is `bad_argument`. Unit tests pin
   the duplicate and bad-middle cases.
5. **`parameters` returns LIVE views** (issue decision 4): `shallow_clone` of
   each param inserted as a tensor entry — same storage, same autograd identity;
   `grad`/`backward` work on them unchanged; later in-place optimizer steps will
   be visible through them. Order: depth-first, weight before bias (PyTorch's
   `.parameters()` order).
6. **`forward` validates kind both ways** (`nn://` module, `tensor://` input —
   the Experiment-1 machinery's first real consumer) and touches both entries.
   Tracked output when params require grad (they do — training is the point).
7. **`nn info`** prints kind, parameter count (tensors and elements), and the
   child kinds for sequential — plain text, one line per fact.
8. **Accounting**: module parameter bytes count in `approx_bytes` (status);
   `torch tensors` stays tensors-only (the listing for modules is a later
   experiment with `nn list` if wanted — recorded).

**Golden strategy (issue design question 2, applied)**: a new golden case type
`nn_linear_forward` — explicit weights from JSON, forward on a known input,
expected output AND expected weight/bias gradients (after `sum().backward()`)
computed by Python `torch.nn.functional.linear` on MPS. Activations and
sequential composition ride the same case type via an op chain. Init parity is
NOT golden-tested (recorded above); seeded-init determinism is a Rust unit test.

## Changes

1. **`nutorchd/src/registry.rs`**: `Object::Module(NnModule)`;
   `insert_module`/`get_module`/`get_module_mut`; `approx_bytes` counts
   parameter tensors; `NnModule` lives in a new `nutorchd/src/nn.rs` (enum,
   `forward`, `parameters`, `param_count`, `describe`).
2. **`nutorchd/src/protocol.rs`**: `Bespoke::Nn { kind, args }`,
   `Bespoke::Forward { module, tensor }`, `Bespoke::NnParameters
   { module }`,
   `Bespoke::NnInfo { module }`.
3. **`nutorchd/src/dispatch.rs`**: the four arms (construction validates
   per-kind args incl. explicit-weight shape checks; forward; parameters as live
   views; info); lease + entry touches per convention. Unit tests: construction
   shapes; explicit-weight shape mismatch errors; seeded-init determinism;
   sequential consumes children (handles gone, forward works); forward
   kind-validation both ways (wrong_kind both directions); parameters are live
   views, proven via grad identity (no in-place op exists yet to mutate through,
   so the shared-autograd-identity property is the testable one: backward
   through `forward` populates gradients readable via the parameter handles —
   impossible unless they alias the module's tensors); sigmoid/tanh/relu forward
   parity with the table ops (gelu has NO table op — verified by review — so
   gelu is checked against its golden instead).
4. **`torch-cli/src/main.rs`**: the `nn` subcommand (kind + args + flags),
   `forward` verb (dual input: tensor from stdin or positional), routing before
   the table path. `nn` kinds and their arg specs live in a small client-side
   match (the module-kind table moves to `ops/` only when the sweep experiment
   needs it — recorded).
5. **`scripts/gen-golden.py` + `nutorchd/tests/golden.rs`**: the
   `nn_linear_forward` case type (forward output + weight/bias grads, explicit
   weights, with and without bias; a sequential linear→relu→linear chain; a gelu
   activation case). The harness gains a bespoke-dispatch helper (via
   `handle_request`) — module construction and `forward` are not table ops, so
   `execute_table` alone cannot reach them. Floor raised to `>= 225`.
6. **`README.md`**: the nn section seed (construction, forward,
   parameters-are-live note) — grows with later experiments.

## Verification

1. **Hygiene**: build 0 warnings; fmt/dprint clean on touched files; full suite
   green; `v1/` untouched.
2. **Goldens**: the new nn cases green (forward + gradients exact vs Python on
   MPS); existing 220 untouched.
3. **Unit tests**: the eight listed in Changes item 3.
4. **Live**:
   - the Description's session verbatim (construct, compose, forward,
     parameters, info);
   - `$x | torch forward $m` (stdin form) equals the positional form;
   - explicit weights: construct `linear --weight $w --bias-tensor $b`, forward
     a known input, `torch value` matches a hand-computed result;
   - gradient flow end to end: forward → `sum` → `backward` →
     `torch grad <param-handle>` is non-empty and exact vs the golden;
   - `torch forward $x $m` (swapped) → `wrong_kind` both directions;
     `torch free $l` then `torch nn sequential $l …` → `unknown handle`
     (consumed/freed children); `torch value $m` → `wrong_kind`;
   - seeded determinism: `manual_seed` → `nn linear 2 3` twice → equal parameter
     values.
5. **Accounting**: `daemon status` bytes grow by the module's parameter bytes on
   construction and shrink on `free $m`.

**Pass** = all five. **Fail** = the own-parameters decision required VarStore
after all, or the op table had to learn about modules.

## Design Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only).
**First pass: CHANGES REQUIRED** — 3 Required, and the first is the kind the
gate exists for: **the VarStore-rejection rested on a factually false claim**
("no supported store merge" — `VarStore::merge` is public and non-deprecated;
the reviewer cited the source line). The decision record was rewritten on the
true ledger (merge exists and even matches our consume semantics; the decision
stands for real reasons: no store/Path bookkeeping, parameter identity under our
control, save/load NOT foreclosed — `Tensor::write_safetensors` covers it,
recorded for that experiment). The other two Required: sequential construction
was not specified atomic (now: validate-all-before-consuming-any, with duplicate
and bad-middle unit tests — the free invariant reapplied), and explicit-weight
semantics were unpinned (now: deep-copy, never alias, requires_grad set LAST
post-copy regardless of source, source unchanged — asserted in test). Optionals
folded: `--bias-tensor`+`--no-bias` and non-linear weight flags error; gelu
parity reworded (no table op exists); the harness's bespoke-dispatch helper
named. The reviewer CONFIRMED the init-fidelity math (kaiming_uniform(a=√5) ≡
U(-1/√in, 1/√in), bias same bound — verified numerically), the shallow_clone
aliasing foundation (same TensorImpl: storage, requires_grad, and .grad shared —
the live-view tests are valid), and client routing collision-freedom.

## Result

**Result:** Pass

The object model is real: `Object::Module` landed with construction,
composition, forward, live parameter views, and info — and the own-parameters
decision survived contact with implementation exactly as re-argued after the
design review's correction.

- **Goldens: 5/5 nn cases first run** (225 total, floor 225; byte-stable at
  sha256 `8324dda3…`) — forward outputs AND weight/bias gradients bitwise-equal
  to `torch.nn.functional` on MPS, with and without bias, through
  relu/gelu/sigmoid chains composed via `sequential`.
- **Unit tests** (67 daemon tests, up from 59): construction shapes +
  seeded-init determinism; explicit weights deep-copied (module param tracks,
  SOURCE tensor's requires_grad unchanged — the pinned semantics);
  shape-mismatch and `--bias-tensor`+`--no-bias` errors; sequential atomicity
  (bad middle AND duplicate child both leave the registry unchanged; happy path
  consumes); forward kind-validation both ways; live views proven by grad
  identity; activation modules whole-tensor-equal to their table ops; unknown
  kind / empty sequential.
- **Live**: the Description session verbatim; stdin forward == positional
  (`equal` true); explicit-weight forward hand-verified (`[[12,23,35]]`); weight
  gradient exact (`[[2,3]]×3`); `wrong_kind` in BOTH directions with messages
  naming both kinds; freed-then-composed child → `unknown handle`; seeded init
  deterministic; accounting walks 448 → 1,051,072 → 448 bytes around a 512×512
  linear — the delta is exactly (512·512+512)·4.
- **Hygiene**: build 0 warnings; fmt/dprint clean; full suite green; `v1/`
  untouched. (Two result-review findings fixed before commit: the CLI's
  `--no-bias` was unreachable — the spec-less flag parser derives hyphenated
  names while the presence-set and the nn match used underscores; both forms now
  accepted and verified live with `nn info` showing `bias: false`. And the
  golden harness's `let mut bespoke` was a new warning falsifying the 0-warnings
  claim — removed.)

## Conclusion

The foundation experiment's promise held: adding `Object::Module` cost one enum
variant, two accessors, and a `nn.rs` of ~120 lines — the typed registry
absorbed a second kind without a single protocol or op-table change. The
live-views contract is proven by gradient identity, the explicit-weight path
doubles as the pretrained-weights loader, and `sequential`'s atomic consume
reuses the free invariant verbatim. Next: losses as table ops (the 0005 loom),
then optimizers + the training-loop acceptance.

## Result Review

**Reviewer:** `adversarial-reviewer` subagent (fresh context, read-only),
reviewing the pre-commit working tree. **First pass: CHANGES REQUIRED** — 2
Required: (1) **the CLI's `--no-bias` was non-functional** — the spec-less flag
parser produces hyphenated names while the presence-set and nn match arm used
underscores, so the design-mandated flag couldn't be expressed through the only
user surface (the daemon side worked; only JSON-direct tests exercised it).
Fixed by accepting both forms, verified live (`nn info` → `bias: false`). (2)
The golden harness's `let mut
bespoke` introduced a new warning, falsifying the
Result's 0-warnings claim — removed, Result corrected to disclose both findings.
Everything else held under attack: all three design-review mandates verified in
code AND behavior (atomic validate-with-duplicate-check before any remove;
deep-copy with requires_grad-last and source-unchanged; the init bound ±1/√2
confirmed numerically), goldens byte-stable with `nn_linear_bias` spot-checked
against fresh `F.linear`, gelu's `"none"` matching Python's default, the
live-views grad identity, `wrong_kind` both directions, and the byte-exact
accounting walk.
