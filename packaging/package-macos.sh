#!/usr/bin/env bash
# A universal macOS binary (arm64 and x86_64 glued with lipo) and its archive.
#   packaging/package-macos.sh [binaries|archive|all]

set -euo pipefail
source "$(dirname "$0")/common.sh"

TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
BINARY="$OUT/bin/universal-apple-darwin/$PROGRAM"

[ "$(uname -s)" = Darwin ] || die "lipo only exists on macOS"

binaries() {
    require_interface
    local installed; installed="$(rustup target list --installed 2>/dev/null || true)"
    for target in "${TARGETS[@]}"; do
        grep -qx "$target" <<<"$installed" \
            || die "the $target standard library is not installed: rustup target add $target"
    done

    for target in "${TARGETS[@]}"; do
        say "building $PROGRAM for $target"
        MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}" \
            cargo build --release --locked --target "$target" --bin "$PROGRAM"
    done

    say "gluing the two into a universal binary"
    mkdir -p "$(dirname "$BINARY")"
    local built="${CARGO_TARGET_DIR:-$ROOT/target}"
    lipo -create \
        "$built/aarch64-apple-darwin/release/$PROGRAM" \
        "$built/x86_64-apple-darwin/release/$PROGRAM" \
        -output "$BINARY"
    chmod 0755 "$BINARY"
    local architectures; architectures="$(lipo -archs "$BINARY")"
    for architecture in arm64 x86_64; do
        grep -qw "$architecture" <<<"$architectures" \
            || die "$BINARY holds $architectures, not both arm64 and x86_64"
    done
    lipo -info "$BINARY"
}

package() {
    [ -x "$BINARY" ] || die "$BINARY is missing; run 'packaging/package-macos.sh binaries' first"
    say "the archive for macOS"
    archive "$BINARY" macos universal home-portal.plist caddy.plist
}

case "${1:-all}" in
binaries) binaries ;;
archive)  package ;;
all)      binaries; package ;;
*) echo "usage: packaging/package-macos.sh [binaries|archive|all]" >&2; exit 2 ;;
esac
