#!/usr/bin/env bash
# A universal macOS binary (arm64 and x86_64 glued with lipo), its archive, and a .pkg that
# installs it and sets the portal up for the user signed in (see packaging/macos/setup).
#   packaging/package-macos.sh [binaries|archive|pkg|check|all]

set -euo pipefail
source "$(dirname "$0")/common.sh"

TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
BINARY="$OUT/bin/universal-apple-darwin/$PROGRAM"
IDENTIFIER="lan.home.portal"
PKG="$RELEASE/${PROGRAM}_${VERSION}.macos.universal.pkg"
MACOS="$ROOT/packaging/macos"

WORK=""
cleanup() { [ -z "$WORK" ] || rm -rf $WORK; }
trap cleanup EXIT

[ "$(uname -s)" = Darwin ] || die "lipo, pkgbuild and productbuild only exist on macOS"

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

built() {
    [ -x "$BINARY" ] || die "$BINARY is missing; run 'packaging/package-macos.sh binaries' first"
}

package() {
    built
    say "the archive for macOS"
    archive "$BINARY" macos universal home-portal.plist caddy.plist
}

stage() {
    local tree="$1"
    built
    install -d -m 0755 "$tree/usr/local/bin" "$tree/usr/local/libexec/home-portal" \
        "$tree/usr/local/share/home-portal" "$tree/usr/local/share/doc/home-portal"
    install -m 0755 "$BINARY" "$tree/usr/local/bin/$PROGRAM"
    install -m 0755 "$ROOT/packaging/provision" "$tree/usr/local/libexec/home-portal/provision"
    install -m 0755 "$MACOS/setup" "$tree/usr/local/libexec/home-portal/setup"
    install -m 0755 "$MACOS/uninstall" "$tree/usr/local/libexec/home-portal/uninstall"
    install -m 0644 "$ROOT/config/home-portal.example.toml" "$tree/usr/local/share/home-portal/home-portal.example.toml"
    install -m 0644 "$ROOT/config/secrets.example.toml" "$tree/usr/local/share/home-portal/secrets.example.toml"
    install -m 0644 "$ROOT/README.md" "$tree/usr/local/share/doc/home-portal/README.md"
    install -m 0644 "$ROOT/LICENSE" "$tree/usr/local/share/doc/home-portal/LICENSE"
    cp -R "$ROOT/examples" "$tree/usr/local/share/doc/home-portal/examples"
    find "$tree/usr/local/share/doc/home-portal/examples" -type d -exec chmod 0755 {} +
    find "$tree/usr/local/share/doc/home-portal/examples" -type f -exec chmod 0644 {} +
    xattr -rc "$tree" 2>/dev/null || true
}

pkg() {
    local work; work="$(mktemp -d)"; WORK="$WORK $work"
    say "staging the install tree"
    stage "$work/root"

    say "pkgbuild"
    COPYFILE_DISABLE=1 pkgbuild \
        --root "$work/root" \
        --identifier "$IDENTIFIER" \
        --version "$VERSION" \
        --scripts "$MACOS/scripts" \
        --ownership recommended \
        --install-location / \
        "$work/home-portal-component.pkg"

    sed -e "s/@VERSION@/$VERSION/g" -e "s/@IDENTIFIER@/$IDENTIFIER/g" \
        "$MACOS/distribution.xml" > "$work/distribution.xml"

    say "productbuild"
    mkdir -p "$RELEASE"
    rm -f "$PKG"
    productbuild \
        --distribution "$work/distribution.xml" \
        --resources "$MACOS/resources" \
        --package-path "$work" \
        "$PKG"
    ls -l "$PKG"
}

check() {
    [ -f "$PKG" ] || die "$PKG is missing; run 'packaging/package-macos.sh pkg' first"
    local work; work="$(mktemp -d)"; WORK="$WORK $work"
    local failed=0
    verdict() {
        if eval "$1"; then printf '  ok    %s\n' "$2"; else printf '  FAIL  %s\n' "$2"; failed=1; fi
    }

    say "what $PKG holds"
    pkgutil --expand "$PKG" "$work/expanded"
    local component="$work/expanded/home-portal-component.pkg"
    mkdir -p "$work/payload"
    (cd "$work/payload" && gzip -dc "$component/Payload" | cpio -i --quiet)
    lsbom -p MUGf "$component/Bom" | awk -F'\t' '$4 !~ /\/\._/' > "$work/bom"
    cat "$work/bom"
    local root="$work/payload/usr/local"
    mode() { awk -F'\t' -v path="./$1" '{ sub(/ +$/, "", $1) } $4 == path { print $1 "\t" $2 "/" $3 }' "$work/bom"; }

    for program in bin/home-portal libexec/home-portal/provision libexec/home-portal/setup \
        libexec/home-portal/uninstall; do
        verdict "[ \"\$(mode usr/local/$program)\" = \"-rwxr-xr-x	root/wheel\" ]" "$program is root's, 0755"
    done
    verdict '[ "$(lipo -archs "$root/bin/home-portal")" = "x86_64 arm64" ]' "the binary is universal"
    verdict '[ -f "$root/share/home-portal/home-portal.example.toml" ]' "the example configuration is shipped"
    verdict '[ -x "$component/Scripts/postinstall" ] && sh -n "$component/Scripts/postinstall"' "postinstall is there and parses"
    verdict 'grep -q "conclusion.html" "$work/expanded/Distribution"' "the installer ends on the page that names the password"

    say "setup for a user, run from the payload against a scratch home"
    local home="$work/home"
    mkdir -p "$home"
    HOME="$home" HOME_PORTAL_PREFIX="$root" HOME_PORTAL_SETUP_NO_START=1 sh "$root/libexec/home-portal/setup"
    local folder="$home/Library/Application Support/home-portal"
    verdict '[ "$(stat -f %Lp "$folder/home-portal.toml")" = 600 ]' "the configuration is 0600"
    verdict '[ "$(stat -f %Lp "$folder/initial-password")" = 600 ]' "initial-password is 0600"
    verdict '[ "$(stat -f %Lp "$folder/scripts")" = 755 ]' "the scripts directory is 0755"
    verdict 'plutil -lint "$home/Library/LaunchAgents/lan.home.portal.plist" >/dev/null' "the LaunchAgent is a valid property list"
    verdict '! grep -q "^\[storage\]" "$folder/home-portal.toml"' "the data lies beside the configuration"
    local before; before="$(shasum -a 256 "$folder/home-portal.toml")"
    HOME="$home" HOME_PORTAL_PREFIX="$root" HOME_PORTAL_SETUP_NO_START=1 sh "$root/libexec/home-portal/setup" >/dev/null
    verdict '[ "$(shasum -a 256 "$folder/home-portal.toml")" = "$before" ]' "running setup again keeps the configuration"

    say "the portal starts and admin signs in"
    local port=18081 password; password="$(cat "$folder/initial-password")"
    HOME_PORTAL_CONFIG="$folder/home-portal.toml" HOME_PORTAL_ADDRESS="127.0.0.1:$port" \
        "$root/bin/home-portal" > "$work/portal.log" 2>&1 &
    local portal=$!
    for _ in $(seq 1 50); do
        curl -fsS "http://127.0.0.1:$port/health" >/dev/null 2>&1 && break
        sleep 0.2
    done
    verdict 'curl -fsS "http://127.0.0.1:$port/health" >/dev/null' "GET /health answers"
    verdict '[ "$(curl -s -o /dev/null -w "%{http_code}" -H "Content-Type: application/json" -d "{\"name\":\"admin\",\"password\":\"$password\"}" "http://127.0.0.1:$port/api/session")" -lt 300 ]' \
        "admin signs in with the password from initial-password"
    kill "$portal"; wait "$portal" 2>/dev/null || true
    [ "$failed" -eq 0 ] || { cat "$work/portal.log"; die "the package is not what it should be"; }
    say "the .pkg is what it should be"
}

case "${1:-all}" in
binaries) binaries ;;
archive)  package ;;
pkg)      pkg ;;
check)    check ;;
all)      binaries; package; pkg; check ;;
*) echo "usage: packaging/package-macos.sh [binaries|archive|pkg|check|all]" >&2; exit 2 ;;
esac
