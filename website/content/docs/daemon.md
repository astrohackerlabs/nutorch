---
title: Native ownership
description: The NuTorch shell owns native tensors, modules and optimizers without a separate daemon.
order: 2
section: Core
---

The unreleased shell replaces the daemon with native values in the main NuTorch
process. This page retains the old daemon URL for existing links.

## Shared ownership

A tensor value holds a shared reference to LibTorch-managed storage. Assigning a
value, putting it in a record or passing it to a function does not copy its data.
Modules own parameters and buffers; optimizers keep live parameter references.
Autograd can retain storage after an intermediate shell variable goes out of scope.

Sequential modules retain shared children. Child aliases remain usable and see
parameter updates and recursive train/eval changes. Duplicate children and
overlapping subtrees are rejected; module construction cannot create cycles.

## Commands that are retired

There is no daemon start, stop, status, socket selection, idle TTL, lease, UUID
lookup, registry-wide tensor listing or free operation. Their old commands fail
without launching an installed legacy executable. Native scope/reference release
replaces manual freeing; existing aliases remain valid.

Display and `torch shape` inspect metadata. `torch value` explicitly downloads
data. `torch nn info` describes a module or optimizer. See
[tensor data conversion](/docs/tensors/) and [module state files](/docs/neural-networks/#state-files).

## Parallel work and interruption

Native operations share one execution gate, including hidden autograd leaves,
views, module buffers and optimizer updates. Parallel Nushell tasks can share
objects safely; kernel submission is serialized. Ctrl+C is checked between
commands and cannot preempt an already-submitted LibTorch kernel.

Exiting NuTorch ends the session. Native values are not cross-process handles.
Use explicit data or safetensors files to retain results.
