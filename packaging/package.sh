#!/usr/bin/env bash
# Static Linux binaries (musl) and their archive, for TARGET:
#   x86_64-unknown-linux-musl (default) or aarch64-unknown-linux-musl.
# rustls brings aws-lc and the embedded interface brings zstd, both C, so the build goes
# through cargo-zigbuild: zig is the C compiler and the linker for either architecture.
#   packaging/package.sh [binaries|archive|all]

set -euo pipefail
source "$(dirname "$0")/common.sh"

TARGET="${TARGET:-x86_64-unknown-linux-musl}"
case "$TARGET" in
x86_64-unknown-linux-musl)  ARCH=x86_64 ;;
aarch64-unknown-linux-musl) ARCH=aarch64 ;;
*) die "no Linux architecture name known for $TARGET" ;;
esac
BINARY="$OUT/bin/$TARGET/$PROGRAM"

binaries() {
    require_interface
    command -v cargo-zigbuild >/dev/null \
        || die "cargo-zigbuild is missing: pip install ziglang cargo-zigbuild"
    rustup target list --installed 2>/dev/null | grep -qx "$TARGET" \
        || die "the $TARGET standard library is not installed: rustup target add $TARGET"

    say "building $PROGRAM for $TARGET"
    cargo zigbuild --release --locked --target "$TARGET" --bin "$PROGRAM"

    mkdir -p "$(dirname "$BINARY")"
    install -m 0755 "${CARGO_TARGET_DIR:-$ROOT/target}/$TARGET/release/$PROGRAM" "$BINARY"
    if command -v file >/dev/null 2>&1; then
        file "$BINARY"
        file "$BINARY" | grep -q 'statically linked\|static-pie' \
            || die "$BINARY is not statically linked"
    fi
}

package() {
    [ -x "$BINARY" ] || die "$BINARY is missing; run 'packaging/package.sh binaries' first"
    say "the archive for $TARGET"
    archive "$BINARY" linux "$ARCH" home-portal.service caddy.service
}

case "${1:-all}" in
binaries) binaries ;;
archive)  package ;;
all)      binaries; package ;;
*) echo "usage: packaging/package.sh [binaries|archive|all]" >&2; exit 2 ;;
esac
