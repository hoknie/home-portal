#!/usr/bin/env bash
# Static Linux binaries (musl), their archive, a deb and an rpm, for TARGET:
#   x86_64-unknown-linux-musl (default) or aarch64-unknown-linux-musl.
# rustls brings aws-lc, which is C, so the build goes
# through cargo-zigbuild: zig is the C compiler and the linker for either architecture.
# The deb needs dpkg-deb (debian:12) and the rpm needs rpmbuild (almalinux:9); both only
# package the binary built before, so they run in those images without Rust.
#   packaging/package.sh [binaries|archive|deb|rpm|all]

set -euo pipefail
source "$(dirname "$0")/common.sh"

TARGET="${TARGET:-x86_64-unknown-linux-musl}"
case "$TARGET" in
x86_64-unknown-linux-musl)  ARCH=x86_64; DEB_ARCH=amd64 ;;
aarch64-unknown-linux-musl) ARCH=aarch64; DEB_ARCH=arm64 ;;
*) die "no Linux architecture name known for $TARGET" ;;
esac
BINARY="$OUT/bin/$TARGET/$PROGRAM"

WORK=""
cleanup() { [ -z "$WORK" ] || rm -rf $WORK; }
trap cleanup EXIT

binaries() {
    require_interface
    command -v cargo-zigbuild >/dev/null \
        || die "cargo-zigbuild is missing: pip install --requirement packaging/requirements.txt"
    local installed; installed="$(rustup target list --installed 2>/dev/null || true)"
    grep -qxF "$TARGET" <<<"$installed" \
        || die "the $TARGET standard library is not installed: rustup target add $TARGET"

    say "building $PROGRAM for $TARGET"
    cargo zigbuild --release --locked --target "$TARGET" --bin "$PROGRAM"

    mkdir -p "$(dirname "$BINARY")"
    install -m 0755 "${CARGO_TARGET_DIR:-$ROOT/target}/$TARGET/release/$PROGRAM" "$BINARY"
    if command -v file >/dev/null 2>&1; then
        local kind; kind="$(file "$BINARY")"
        printf '%s\n' "$kind"
        grep -q 'statically linked\|static-pie' <<<"$kind" \
            || die "$BINARY is not statically linked"
    fi
}

built() {
    [ -x "$BINARY" ] || die "$BINARY is missing; run 'packaging/package.sh binaries' first"
}

package() {
    built
    say "the archive for $TARGET"
    archive "$BINARY" linux "$ARCH" home-portal.service caddy.service
}

stage() {
    local tree="$1"
    built
    install -d -m 0755 "$tree/usr/bin" "$tree/usr/lib/home-portal" "$tree/usr/share/home-portal" \
        "$tree/usr/lib/systemd/system" "$tree/usr/lib/sysusers.d" "$tree/usr/lib/tmpfiles.d" \
        "$tree/usr/share/doc/home-portal"
    install -m 0755 "$BINARY" "$tree/usr/bin/$PROGRAM"
    install -m 0755 "$ROOT/packaging/provision" "$tree/usr/lib/home-portal/provision"
    install -m 0755 "$ROOT/packaging/linux/after-install" "$tree/usr/lib/home-portal/after-install"
    install -m 0755 "$ROOT/packaging/linux/before-remove" "$tree/usr/lib/home-portal/before-remove"
    install -m 0644 "$ROOT/packaging/linux/home-portal.service" "$tree/usr/lib/systemd/system/home-portal.service"
    install -m 0644 "$ROOT/packaging/linux/sysusers.conf" "$tree/usr/lib/sysusers.d/home-portal.conf"
    install -m 0644 "$ROOT/packaging/linux/tmpfiles.conf" "$tree/usr/lib/tmpfiles.d/home-portal.conf"
    install -m 0644 "$ROOT/config/home-portal.example.toml" "$tree/usr/share/home-portal/home-portal.example.toml"
    install_interface "$tree/usr/share/home-portal/web"
    install -m 0644 "$ROOT/config/secrets.example.toml" "$tree/usr/share/home-portal/secrets.example.toml"
    install -m 0644 "$ROOT/README.md" "$tree/usr/share/doc/home-portal/README.md"
    install -m 0644 "$ROOT/LICENSE" "$tree/usr/share/doc/home-portal/copyright"
    cp -R "$ROOT/examples" "$tree/usr/share/doc/home-portal/examples"
    find "$tree/usr/share/doc/home-portal/examples" -type d -exec chmod 0755 {} +
    find "$tree/usr/share/doc/home-portal/examples" -type f -exec chmod 0644 {} +
}

deb() {
    command -v dpkg-deb >/dev/null || die "no dpkg-deb here; run this in debian:12"
    local tree; tree="$(mktemp -d)"; WORK="$WORK $tree"
    say "the deb for $TARGET"
    stage "$tree"
    install -d -m 0755 "$tree/DEBIAN"
    local size; size="$(du -k -s "$tree" | cut -f1)"
    sed -e "s/@VERSION@/$VERSION/" -e "s/@ARCH@/$DEB_ARCH/" -e "s/@INSTALLED_SIZE@/$size/" \
        "$ROOT/packaging/deb/control.in" > "$tree/DEBIAN/control"
    for script in postinst prerm postrm; do
        install -m 0755 "$ROOT/packaging/deb/$script" "$tree/DEBIAN/$script"
    done
    ( cd "$tree" && find . -path ./DEBIAN -prune -o -type f -print0 \
        | xargs -0 md5sum | sed 's| \./| |' > DEBIAN/md5sums )
    chmod 0644 "$tree/DEBIAN/md5sums"

    mkdir -p "$RELEASE"
    local target="$RELEASE/${PROGRAM}_${VERSION}.debian.${DEB_ARCH}.deb"
    rm -f "$target"
    dpkg-deb --root-owner-group --build "$tree" "$target"
    dpkg-deb --info "$target"
    ls -l "$target"
}

rpm() {
    command -v rpmbuild >/dev/null || die "no rpmbuild here; run this in almalinux:9"
    local tree; tree="$(mktemp -d)"
    local build; build="$(mktemp -d)"
    WORK="$WORK $tree $build"
    say "the rpm for $TARGET"
    stage "$tree"
    rpmbuild -bb "$ROOT/packaging/rpm/home-portal.spec" \
        --target "$ARCH" \
        --define "portal_version $VERSION" \
        --define "_sourcedir $tree" \
        --define "_topdir $build" \
        --define "_rpmdir $build/rpms" \
        --define "_build_id_links none" \
        --define "dist %{nil}"

    local built; built="$(find "$build/rpms" -name "${PROGRAM}-${VERSION}-1.${ARCH}.rpm" -print -quit)"
    [ -n "$built" ] || die "rpmbuild produced no ${PROGRAM}-${VERSION}-1.${ARCH}.rpm"
    mkdir -p "$RELEASE"
    local target="$RELEASE/${PROGRAM}_${VERSION}.el.${ARCH}.rpm"
    install -m 0644 "$built" "$target"
    command rpm -qpi "$target"
    ls -l "$target"
}

case "${1:-all}" in
binaries) binaries ;;
archive)  package ;;
deb)      deb ;;
rpm)      rpm ;;
all)      binaries; package ;;
*) echo "usage: packaging/package.sh [binaries|archive|deb|rpm|all]" >&2; exit 2 ;;
esac
