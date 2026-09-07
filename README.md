# NuTorch

PyTorch-style GPU tensor operations from any shell, maintained by Astrohacker.
The `nutorchd` daemon owns tensors and autograd state on Apple-silicon MPS;
`torch` is its thin CLI, also installed as `nutorch`. The Nushell module adds
native lists, records and pipeline/argument parity.

## Install

```nu
brew tap astrohackerlabs/nutorch
brew trust astrohackerlabs/nutorch
brew install astrohackerlabs/nutorch/nutorch
```

Upgrade with `brew update` then
`brew upgrade astrohackerlabs/nutorch/nutorch`. Uninstall with
`brew uninstall astrohackerlabs/nutorch/nutorch`. NuTorch is independent of
Astrohacker TermSurf. The tap is
[astrohackerlabs/homebrew-nutorch](https://github.com/astrohackerlabs/homebrew-nutorch).

Read the [NuTorch documentation](https://nutorch.com/docs/) for operation
reference, tensor lifecycle, neural networks and shell examples. Tensors live
in the daemon; stopping it discards its tensor registry. Do not stop a daemon
that owns work you need to retain.

## Source

This standalone source is exported from the Astrohacker development repository.
Rust workspace members are `nutorchd`, `torch-cli`, and `ops`; the generated
Nushell client is `nutorch.nu`. Website source is in `website/` and builds with
`bun install --frozen-lockfile` followed by `bun run build` there.

For a Homebrew-managed source build, use:

```nu
brew install --build-from-source astrohackerlabs/nutorch/nutorch
```

Direct Cargo development needs the pinned LibTorch 2.11.0 headers and libraries
at `.libtorch/` in this workspace. Run Cargo from this directory so its local
configuration is loaded. `cargo test --locked --workspace` includes real MPS
tests and requires Apple silicon. Do not substitute a CPU-only test result.

Release and deployment operator scripts are maintained in the Astrohacker
repository, not this source mirror. The MIT license is in `LICENSE`; bundled
LibTorch license and notices are in `legal/libtorch/`.
