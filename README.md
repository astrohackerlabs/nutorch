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

Homebrew builds from the published source archive:

```nu
brew tap astrohackerlabs/nutorch git@github.com:astrohackerlabs/homebrew-nutorch.git
brew trust astrohackerlabs/nutorch
brew install astrohackerlabs/nutorch/nutorch
```

For an existing installation, run `brew update` and
`brew upgrade astrohackerlabs/nutorch/nutorch`. Check `nutorch --version`;
1.x is the earlier tensor-tool product. Start `nutorch` interactively or run
ordinary Nushell scripts with `nutorch script.nu`.

## Source layout

The Rust workspace contains `shell`, `core` and `ops`. `forks/` contains verified
Nushell/Reedline sources and licenses; `source-provenance.json` records pins and
export transformations. LibTorch notices live in `legal/libtorch/`.

To build, place the `torch` directory from the formula's exact SHA-pinned
LibTorch wheel at `.libtorch` (or symlink it there), then run
`cargo build --locked --release --bin nutorch` from this directory.
The formula records the resource URL, checksum, runtime dylibs and installation
layout. A build alone does not install the shell. `website/` contains standalone
Bun/React Router source; run `bun install --frozen-lockfile` and `bun run build`
there. Website deployment is independent of the shell release.

NuTorch is independently versioned. TermSurf dependency/default-shell integration
is pending. No release script may install or upgrade Homebrew packages.
