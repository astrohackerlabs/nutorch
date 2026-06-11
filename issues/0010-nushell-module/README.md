+++
status = "open"
opened = "2026-06-11"
+++

# Issue 10: The Nushell module — native data over the same thin CLI

## Goal

Nushell users work with nutorch in native structured data — lists in, tables
out, listings queryable — through a generated `torch.nu` wrapper module over the
existing CLI, plus a `--json` output mode on the structured verbs:

```nu
use nutorch.nu *

let t = [[1 2] [3 4]] | nutorch tensor
$t | nutorch mm $t | nutorch value           # a native table
nutorch tensors | where bytes > 1mb | get handle | each {|h| nutorch free $h }
```

## Background — why a module, not a plugin

A Nushell PLUGIN was considered and rejected (recorded from design discussion):

- **The nu-plugin API treadmill**: the plugin protocol is versioned to Nushell
  releases and churns with them — a plugin is a standing maintenance commitment
  to track every Nushell version. A `.nu` module of external-command wrappers
  sits on Nushell's most stable surface.
- **Architectural consistency**: the project's principle is "the daemon owns
  everything; clients are thin." A generated script module is the thinnest
  possible client. The plugin's genuine advantages — a persistent socket
  connection (performance) and custom value types — are recorded as what was
  traded away; performance remains a separate measure-first backlog item.

The wrapper fixes exactly the friction Nushell has with the plain CLI: the data
boundary. `$in | to json` going in; `from json` coming out; the non-finite
dialect tokens (`"NaN"`/`"Infinity"`/`"-Infinity"`) mapped to Nushell's REAL
float NaN/infinity — making round-trips cleaner in Nushell than in bash.

## Decisions Already Made

1. **`nutorch.nu` is GENERATED from the ops table.** A `torch nu-module` CLI
   verb emits a wrapper per table op from the same `OpSpec` rows that drive the
   daemon, the CLI grammar, and `torch ops` — the fourth consumer of the single
   source of truth; it cannot drift. Bespoke verbs (tensor, value, free,
   tensors, nn family, forward, step, daemon) get hand-written wrappers in a
   static prelude the generator includes.
2. **`--json` output mode** on the structured verbs — `tensors`, `ops`,
   `nn info`, `daemon status` — emitting the wire JSON the CLI already holds
   before it renders text. Useful beyond Nushell (jq users).
3. **Handles stay plain strings** (`tensor://…`) — the typed prefixes were
   designed for exactly this; no custom value type.
4. **No daemon changes.** The wire protocol is untouched; the `--json` flag is
   client-side rendering. (The mandate boundary, as in issues 0007/0009: daemon
   changes mean the issue exceeded its scope.)

## Scope

In: the `--json` flag; the `torch nu-module` generator + prelude (covering every
table op and every bespoke verb, with input conversion, output conversion,
NaN-token mapping, and wrapper signatures that carry flags faithfully); a
committed convenience copy of the generated `torch.nu`; docs (README section); a
Nushell verification session exercising the core workflows natively — tensor
round-trips, the census-query-free composition, autograd, and a training loop.

Out (recorded): a nu-plugin (rejected above); persistent-connection performance
(the separate backlog item); custom values; Nushell completions beyond what
wrapper signatures give for free.

## Design Questions (settled per-experiment)

1. **Command naming — SETTLED (user decision)**: subcommand style under a
   distinct `nutorch` namespace (`def "nutorch tensor"`, `nutorch nn
   linear`,
   …), perfectly analogous to the CLI's `torch tensor`. Flat `torch-tensor`
   rejected as clumsy; same-name `def "torch tensor"` rejected for PARTIAL
   SHADOWING — wrapped subcommands would be native while unwrapped ones silently
   fall through to the external binary, two behaviors under one name. The module
   file is `nutorch.nu`.
2. **Flag fidelity in wrappers**: Bool presence flags, HandleOrScalar, IntList —
   how each ParamKind maps to a Nushell wrapper parameter.
3. **Where the committed `nutorch.nu` lives** and how staleness is guarded (a
   test that regenerates and diffs, like the golden byte-stability check).
4. **Nushell availability for verification**: is `nu` on this machine, or does
   the verification install/pin one? The experiment must say.

## Experiments

- [Experiment 1: `--json`, the generator, and the module](01-nu-module.md) —
  **Designed**
