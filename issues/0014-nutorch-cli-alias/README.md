+++
status = "open"
opened = "2026-06-12"
+++

# Issue 14: `nutorch` on the command line

## Goal

`nutorch` works as a CLI command everywhere `torch` does — same binary, both
names. The project is CALLED nutorch; a user who types its name should get the
tool.

## Background

Today the surface is split: the binary is `torch` (PyTorch fidelity — the
muscle-memory name), while `nutorch` exists only as the Nushell module namespace
(`use nutorch.nu *` → `nutorch tensor`, …). Outside Nushell, `nutorch` is
command-not-found. The user hit exactly this confusion (2026-06-12), compounded
by a stale v1 plugin registration shadowing `torch` inside Nushell (now
removed).

The client locates its daemon as a SIBLING of its own executable
(`torch-cli/src/main.rs`: `current_exe().parent().join("nutorchd")`, with
`NUTORCHD_BIN` as override) — any aliasing mechanism must keep that working.

## Analysis

Two mechanisms considered:

1. **A second `[[bin]]` in torch-cli** — produces a real duplicate binary. Costs
   a second copy of the client in every install and keg for zero behavioral
   difference.
2. **A symlink at install time** (`nutorch` → `torch`) — the standard Unix
   answer (`python3`/`python`, `vim`/`vi`). Same directory, so the sibling
   daemon lookup is unaffected whether or not the OS resolves the link; nothing
   in the client reads `argv[0]`.

Symlink wins. It lands in: `scripts/install.sh` (from-source installs),
`dist/nutorch.rb` (`bin.install_symlink` — brew links it into
`$(brew --prefix)/bin` like any binstub), and the docs that enumerate what gets
installed.

Inside Nushell, after `use nutorch.nu *`, the module's `nutorch` commands shadow
the external — which is correct: the module IS the richer Nushell client, and it
calls `^torch` underneath.

## Experiments

- [Experiment 1: The symlink](01-the-symlink.md) — **Designed**

## Scope

In: the symlink in `install.sh` and `dist/nutorch.rb`; docs touch
(install-from-source page, getting-started/README one-liners where the binary
names are listed); a locally created symlink so the user's existing brew install
gains `nutorch` immediately.

Out (recorded): updating the published tap formula and re-bottling — the
existing v0.1.0 bottle cannot grow a symlink retroactively; the tap picks the
change up with the NEXT release (same precedent as the license metadata);
argv[0]-based behavior differences (none exist, none added).
