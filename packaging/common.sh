# Sourced by the packaging scripts: the version, the release folder and the archive layout.

cd "$(dirname "${BASH_SOURCE[0]}")/.."
ROOT="$PWD"
OUT="${OUT:-$ROOT/dist}"
RELEASE="${RELEASE:-$OUT/release}"
PROGRAM="home-portal"

VERSION="${VERSION:-$(awk '/^\[workspace\.package\]/{p=1} p && /^version *= *"/{gsub(/[^0-9A-Za-z.+~-]/,"",$3); print $3; exit}' Cargo.toml)}"
[ -n "$VERSION" ] || { echo "packaging: cannot read the version out of Cargo.toml" >&2; exit 2; }

say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }
die() { printf 'packaging: %s\n' "$*" >&2; exit 1; }

require_interface() {
    [ -f "$ROOT/web/out/en/index.html" ] \
        || die "web/out is missing; the binary embeds the interface, so run 'just web' first"
}

# archive <binary> <platform> <arch> <deploy files...>
# Writes $RELEASE/home-portal_<version>.<platform>.<arch>.tar.gz holding the binary, the
# configuration examples, the examples folder and the deploy files beside the binary.
archive() {
    local binary="$1" platform="$2" arch="$3"
    shift 3
    local name="${PROGRAM}_${VERSION}.${platform}.${arch}"
    local work; work="$(mktemp -d)"
    local tree="$work/$name"

    install -d -m 0755 "$tree" "$tree/config"
    install -m 0755 "$binary" "$tree/$PROGRAM"
    install -m 0644 "$ROOT/config/home-portal.example.toml" "$tree/config/home-portal.example.toml"
    install -m 0644 "$ROOT/config/secrets.example.toml" "$tree/config/secrets.example.toml"
    cp -R "$ROOT/examples" "$tree/examples"
    find "$tree/examples" -type d -exec chmod 0755 {} +
    find "$tree/examples" -type f -exec chmod 0644 {} +
    find "$tree/examples" -name '*.sh' -exec chmod 0755 {} +
    for deploy in "$@"; do
        install -m 0644 "$ROOT/examples/deploy/$deploy" "$tree/$deploy"
    done
    install -m 0644 "$ROOT/README.md" "$tree/README.md"
    install -m 0644 "$ROOT/LICENSE" "$tree/LICENSE"

    mkdir -p "$RELEASE"
    local target="$RELEASE/$name.tar.gz"
    rm -f "$target"
    local flavour; flavour="$(tar --version 2>/dev/null || true)"
    if grep -q GNU <<<"$flavour"; then
        tar --owner=0 --group=0 --numeric-owner --sort=name --mtime=@0 \
            -C "$work" -cf - "$name" | gzip -9 -n > "$target"
    else
        COPYFILE_DISABLE=1 tar -C "$work" --uid 0 --gid 0 --uname root --gname wheel \
            -cf - "$name" | gzip -9 -n > "$target"
    fi
    rm -rf "$work"

    local listing; listing="$(tar -tzf "$target")"
    grep -qxF "$name/$PROGRAM" <<<"$listing" || die "$target does not hold $name/$PROGRAM"
    ls -l "$target"
}
