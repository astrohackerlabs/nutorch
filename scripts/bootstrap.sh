#!/bin/zsh
# Nutorch bootstrap (issue 0011): one command from fresh checkout to built
# release binaries. Idempotent — skips the torch download when .venv-torch
# already holds the pinned version. Run from anywhere inside the repo.
set -e
cd "$(dirname "$0")/.."

TORCH_VERSION=2.11.0

if [ -x .venv-torch/bin/python ] && \
   .venv-torch/bin/python -c "import torch, sys; sys.exit(0 if torch.__version__.startswith('$TORCH_VERSION') else 1)" 2>/dev/null; then
  echo "bootstrap: .venv-torch already has torch $TORCH_VERSION (skipping download)"
else
  echo "bootstrap: creating .venv-torch with torch $TORCH_VERSION"
  python3 -m venv .venv-torch
  .venv-torch/bin/pip install --quiet "torch==$TORCH_VERSION"
fi

# The .libtorch symlink points at the venv's torch package (the libtorch
# dylibs + headers torch-sys builds against).
TORCH_PKG=$(.venv-torch/bin/python -c "import torch, os; print(os.path.dirname(torch.__file__))")
if [ "$(readlink .libtorch 2>/dev/null)" != "$TORCH_PKG" ]; then
  rm -f .libtorch
  ln -s "$TORCH_PKG" .libtorch
  echo "bootstrap: linked .libtorch -> $TORCH_PKG"
fi

echo "bootstrap: building (release)"
cargo build --release
echo "bootstrap: done — binaries in target/release; install with scripts/install.sh"
