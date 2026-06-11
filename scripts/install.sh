#!/bin/zsh
# Nutorch install (issue 0011): copy release binaries + the required
# libtorch dylib subset + nutorch.nu into a prefix (default ~/.nutorch).
# The binaries' baked rpath @loader_path/../libexec/libtorch/lib resolves
# the dylibs keg-relative, so the install is relocatable.
set -e
cd "$(dirname "$0")/.."

PREFIX="${1:-$HOME/.nutorch}"

for bin in torch nutorchd; do
  [ -x "target/release/$bin" ] || { echo "install: target/release/$bin missing — run scripts/bootstrap.sh first" >&2; exit 1; }
done

# The measured dylib closure (issue 0011 exp 1): nutorchd links
# libtorch/libtorch_cpu/libc10 by @rpath; libtorch_cpu additionally
# references @rpath/libomp.dylib — libomp is REQUIRED despite not being a
# direct link. libtorch_python/libshm/libtorch_global_deps are unreferenced.
DYLIBS=(libtorch.dylib libtorch_cpu.dylib libc10.dylib libomp.dylib)

mkdir -p "$PREFIX/bin" "$PREFIX/libexec/libtorch/lib" "$PREFIX/share/nutorch"
cp target/release/torch target/release/nutorchd "$PREFIX/bin/"
for dylib in $DYLIBS; do
  cp ".libtorch/lib/$dylib" "$PREFIX/libexec/libtorch/lib/"
done
cp nutorch.nu "$PREFIX/share/nutorch/nutorch.nu"

echo "installed to $PREFIX"
echo "  binaries:   $PREFIX/bin (add to PATH)"
echo "  nushell:    use $PREFIX/share/nutorch/nutorch.nu *"
"$PREFIX/bin/torch" --version
