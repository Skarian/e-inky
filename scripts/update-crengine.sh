#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRENGINE_DIR="$ROOT_DIR/crates/crengine"
VENDOR_DIR="$CRENGINE_DIR/vendor"
PATCH_DIR="$CRENGINE_DIR/patches"

UPSTREAM_REPO="${UPSTREAM_REPO:-https://gitlab.com/coolreader-ng/crengine-ng.git}"
UPSTREAM_COMMIT="${UPSTREAM_COMMIT:-054875c021539c21e93665fcfc969d61d5a3e9e8}"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

echo "Fetching CREngine-NG from ${UPSTREAM_REPO} @ ${UPSTREAM_COMMIT}"
git clone --filter=blob:none "$UPSTREAM_REPO" "$tmp_dir/crengine-ng"
git -C "$tmp_dir/crengine-ng" checkout "$UPSTREAM_COMMIT"

echo "Syncing sources into ${VENDOR_DIR}"
mkdir -p "$VENDOR_DIR"
rsync -a --delete --exclude='.git' "$tmp_dir/crengine-ng/" "$VENDOR_DIR/"

shopt -s nullglob
for patch in "$PATCH_DIR"/*.patch; do
  echo "Applying patch $(basename "$patch")"
  (cd "$VENDOR_DIR" && git apply "$patch")
done
shopt -u nullglob

echo "Done. Vendored sources live in ${VENDOR_DIR} (ignored by Git)."
