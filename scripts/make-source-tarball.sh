#!/bin/zsh
# Issue 0011: build the source tarball that ships as the immutable GitHub
# Release asset (Experiment 3). The formula's url points at the uploaded
# asset, so there is no sha-patching here — the sha is read off the artifact
# and written into the tap formula at release time.
#
# Usage: make-source-tarball.sh [ref]   (default: HEAD)
set -e
cd "$(dirname "$0")/.."

REF=${1:-HEAD}
VERSION=$(rg -o 'version = "([0-9.]+)"' -r '$1' Cargo.toml | head -1)
OUT=/tmp/nutorch-src
mkdir -p $OUT
git archive --format=tar.gz --prefix="nutorch-$VERSION/" -o "$OUT/nutorch-$VERSION.tar.gz" "$REF"
SHA=$(shasum -a 256 "$OUT/nutorch-$VERSION.tar.gz" | awk '{print $1}')
echo "tarball: $OUT/nutorch-$VERSION.tar.gz ($REF)"
echo "sha256:  $SHA"
