+++
status = "closed"
opened = "2026-06-11"
closed = "2026-06-11"
+++

# Issue 9: nn/optim — modules and optimizers as first-class objects

## Goal

The complete nn/optim surface, object-style: modules and optimizers are
daemon-resident objects with their own handles, composed and trained from the
shell in PyTorch's own shape:

```bash
l1=$(torch nn linear 1 16)
model=$(torch nn sequential $l1 "$(torch nn relu)" "$(torch nn linear 16 1)")
opt=$(torch nn sgd $model --lr 0.01)
for i in $(seq 200); do
  pred=$(torch forward $model $x)
  loss=$(torch mse_loss $pred $y)
  torch backward $loss
  torch step $opt
  torch nn zero_grad $opt
done
```

This issue is deliberately large — the issue-0005 pattern ("all tensor ops": one
issue, sweep experiments designed one at a time) applied to the nn/optim
surface.

## Decisions Already Made (recorded from design discussion)

1. **Object approach, not functional.** Modules are daemon-side objects (the
   only home for optimizer state like Adam's moments, train/eval mode, and
   nested composition). This was weighed against a functional-only style and
   chosen deliberately.
2. **One registry, typed entries.** Tensors, modules, and optimizers live in the
   SAME registry behind the same lock, as variants of a typed entry. Chosen for
   error quality (a single lookup can say "handle is a module, expected a
   tensor" — separate tables would lie with "unknown handle") and because all
   the issue-0006 machinery (free, listing, accounting) then generalizes instead
   of duplicating.
3. **Typed handle scheme, uniform.** Handles become `tensor://<uuid>`,
   `nn://<uuid>`, `optim://<uuid>`. Rationale: in Python the type system says
   what an object is; in the shell the STRING is all there is — handles live in
   env vars, scripts, logs, and listings, and must carry their own kind.
   Implementation contract: mint with prefix, key by UUID internally, kind-check
   on lookup — so a wrong prefix on a real object reports the true kind, and
   only genuinely absent handles say "unknown handle". **Uniformity is
   non-negotiable**: existing tensor handles migrate to `tensor://` in the same
   issue (pre-release, one mechanical sweep; now is the last cheap moment).
4. **Live parameter views.** `torch nn parameters $model` returns tensor handles
   that ALIAS module weights; `torch step` mutates them in place. This is the
   first deliberately mutable handle state in nutorch — it is the module
   semantic, PyTorch-faithful, and gets loud documentation.

## Scope — what "all" means

- **Foundation**: the typed registry + handle scheme migration (every existing
  test, golden, and doc updated in one sweep); wrong-kind errors;
  `free`/listing/accounting over all kinds.
- **Modules** (the oracle decides MPS survivors, per the issue-0005 pattern):
  linear, conv1d/2d (+ transpose), embedding, layer_norm, batch_norm (1d/2d),
  group_norm, dropout, the activation modules (relu, sigmoid, tanh, gelu,
  leaky_relu, softmax), max_pool2d/avg_pool2d, flatten, sequential, and the
  recurrent family (lstm, gru) if tch + MPS cooperate — exclusions recorded,
  never worked around.
- **Module verbs**: `forward` (dual input pattern), `nn parameters`,
  `nn train`/`nn eval`, `nn info` (kind, parameter count, training mode),
  save/load (the one place shell redirection cannot substitute — a nested
  module's state_dict; format decided in its experiment, likely safetensors via
  tch).
- **Losses as table ops**: mse_loss, l1_loss, cross_entropy, nll_loss,
  binary_cross_entropy (+ with_logits), smooth_l1, huber, kl_div — ordinary
  tensor→tensor rows on the issue-0005 loom.
- **Optimizers**: sgd (momentum, weight_decay, nesterov), adam, adamw, rmsprop —
  whatever tch exposes that runs; per-optimizer flags faithful to torch.optim
  defaults. Verbs: `step`, `nn zero_grad`, `nn set_lr`.
- **The end-to-end acceptance**: a plain shell script trains a small regression
  AND a small classification to verifiably decreasing loss — the issue's reason
  to exist, demonstrated.

Out (recorded): distributed anything, transformer convenience modules beyond
what tch::nn ships, learning-rate schedulers (a recorded follow-up — they are
pure optimizer-verb sugar once `set_lr` exists), custom autograd functions.

## Design Questions (settled per-experiment, not here)

1. **tch `nn::VarStore` vs. own parameter management.** VarStore +
   `nn::Optimizer` is tch's idiomatic path and gives optimizers for free, but
   VarStore wants to own a model's variables at construction — which fights
   handle-style composition (children built first, composed later). The
   alternative: modules hold tensors directly, `sequential` CONSUMES its
   children's handles (documented), and optimizers are our own structs over
   shallow-cloned parameter references (tch tensors are refcounted; in-place
   steps propagate). The foundation experiment must prototype both far enough to
   choose with evidence.
2. **Golden strategy for modules.** Matching Python's weight-init RNG is a
   losing game; goldens load EXPLICIT weights into modules and verify forward
   outputs and gradients exactly against Python on MPS. Init determinism
   (seeded, reproducible daemon-side) is tested separately.
3. **Listings**: does `torch tensors` stay tensors-only with an `nn list`
   sibling, or grow a kind column? (Leaning: `tensors` keeps its name and
   contract; objects get their own census.)
4. **Construction grammar**: `torch nn <kind> <args>` subcommand style (mirrors
   `torch.nn.*`), with per-kind positional/flag specs — whether these ride the
   existing op table or a parallel module table is the foundation experiment's
   call.
5. **Dropout/RNG**: the seeded-CPU-generator convention (issue 0005) vs. MPS
   dropout kernels — determinism contract decided when dropout lands.

## Verification Posture

Same as every issue: hygiene gates, adversarial review on every design and every
result, goldens against the exact linked PyTorch on MPS, oracle-decided
exclusions recorded. The issue closes only when the training-loop acceptance
script runs end to end and the full module/ optimizer scope above is delivered
or honestly excluded.

## Experiments

- [Experiment 1: Typed handles and the typed registry](01-typed-handles.md) —
  **Pass** (every handle now tensor://…; the error quartet live; goldens
  byte-identical; zero client/protocol/ops changes)
- [Experiment 2: The module foundation — linear, activations, sequential, forward](02-module-foundation.md)
  — **Pass** (Object::Module for one enum variant; 5/5 nn goldens first run;
  live views proven by grad identity; sequential consumes atomically)
- [Experiment 3: Losses as table ops](03-losses.md) — **Pass** (nine losses,
  12/12 goldens first run, zero exclusions; the loss→backward→weight-grad path
  live)
- [Experiment 4: Optimizers and the training loop](04-optimizers.md) — **Pass**
  (4 optimizers bitwise vs torch.optim incl. the lerp_-pinning coupled-wd Adam;
  BOTH acceptance scripts train successfully from plain zsh — the issue's goal
  demonstrated)
- [Experiment 5: The module sweep — conv, norms, dropout, embedding, pools](05-module-sweep.md)
  — **Pass** (13 kinds, 13/13 goldens first-run; one recorded ULP exclusion
  (group_norm C-API vs Python dispatch); train/eval mode; training regression
  unchanged)
- [Experiment 6: Save and load — the state_dict for nested modules](06-save-load.md)
  — **Pass** (round trip exact incl. buffers; PyTorch state_dict keys verbatim;
  Python reads the files; optimizer aliasing survives load)

## Conclusion

**Solved.** Six experiments, six passes, twelve adversarial review gates — the
largest issue since 0005, and the one that completes the project's original
wishlist (tensor ops, concurrency, autograd, nn/optim).

Delivered, per the Goal's own example which now runs verbatim:

1. **The typed foundation** — `tensor://`/`nn://`/`optim://` handles over one
   kind-checked registry; wrong-kind errors name what stands behind a handle.
2. **19 module kinds** — linear, the conv family, embedding, the three norms,
   dropout (own-mask, seeded, p-edge-safe), six activations, pools, flatten,
   sequential (atomic consume) — with train/eval mode, live parameter views
   (proven by gradient identity), explicit-weight loading, and PyTorch-default
   seeded init. lstm/gru excluded on the recorded multi-output-forward grounds.
3. **Nine losses** as ordinary table rows.
4. **Four optimizers** whose 3-step trajectories are BITWISE equal to
   `torch.optim` on MPS — including the design review's empirical discovery that
   PyTorch's Adam uses `lerp_` where the textbook diverges by 1 ULP under
   coupled weight decay.
5. **The acceptance**: two committed zsh scripts that train a regression (loss
   6.0 → 2.5e-7) and a classifier (100%) on the GPU — the issue's reason to
   exist, demonstrated and re-verified at every later gate.
6. **state_dict save/load** — safetensors with PyTorch's exact key scheme
   including `num_batches_tracked`, interchange verified in BOTH directions,
   optimizer aliasing surviving load.

Suite at close: 79 daemon unit tests, 255 golden vectors, 3 smoke — all green; 0
warnings; recorded exclusions: lstm/gru (contract), group_norm's exact golden (a
1-ULP C-API-vs-Python dispatch divergence, pinned by an internal-consistency
test instead).

The review gates caught, among others: a false VarStore claim corrected with
source evidence; the `.to()` non-leaf trap that would have orphaned every random
weight's gradients; the Adam `lerp_` ULP; a divide-by-zero daemon-crasher in
conv groups; the dropout p=1 NaN; the missing `num_batches_tracked` that would
have broken interchange both ways; and a silently-unreachable `--no-bias` flag.
Every one was found BEFORE its result commit.
