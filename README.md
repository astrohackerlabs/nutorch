# NuTorch

NuTorch is a Nushell-based shell with built-in GPU tensors, autograd, neural
networks and optimizers. Native values share ownership in the main process;
LibTorch manages Apple-silicon MPS storage.

The 2.0 source tree replaces the earlier tensor tools with a native shell.
It requires Apple-silicon macOS. Shift+Tab opens an unfinished AI input mode;
it does not translate or execute an AI request.

The only runtime executable is `nutorch`. Import its native commands with `use torch`.
The daemon, external tensor CLI and imported Nushell client are removed.

```nu
use torch
let x = torch tensor [1 2 3] --requires-grad
$x | torch mul $x | torch sum | torch backward
$x | torch grad | torch value
```

## Installation

The 2.0.2 binary distribution includes the NuTorch executable and its LibTorch
runtime for Apple-silicon macOS Tahoe with standard Homebrew (`/opt/homebrew`).
Normal installation downloads the prebuilt archive; no Rust compiler, Python or
separate LibTorch installation is required. Other targets are not yet qualified.

```nu
brew trust astrohackerlabs/astrohacker
brew tap astrohackerlabs/astrohacker
brew install nutorch
```

For an existing installation, run `brew update` and
`brew upgrade nutorch`. Check `nutorch --version`;
1.x is the earlier tensor-tool product. Start `nutorch` interactively or run
ordinary Nushell scripts with `nutorch script.nu`.

## Source layout

The Rust workspace contains `shell`, `core` and `ops`. `forks/` contains verified
Nushell/Reedline sources and licenses; `source-provenance.json` records pins and
export transformations. LibTorch notices live in `legal/libtorch/`.

To build from source, place the `torch` directory from PyTorch 2.11.0's
Apple-silicon wheel at `.libtorch` (or symlink it there), then run
`cargo build --locked --release --bin nutorch` from this directory.
The binary distribution bundles libtorch, libtorch_cpu, libc10 and libomp.
A source build alone does not install the shell. `website/` contains standalone
Bun/React Router source; run `bun install` and `bun run build`
there. Website deployment is independent of the shell release.

NuTorch is independently versioned. TermSurf dependency/default-shell integration
is pending. Producer builds use the normal workspace cache; publication and
consumer installation are separate operations.
