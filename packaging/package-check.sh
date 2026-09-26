#!/usr/bin/env bash
# Installs the deb (in debian:12) or the rpm (in almalinux:9) of this version and TARGET on a
# clean system, checks what the installation made, runs the portal as its user, signs in as
# admin with the password the installation wrote, then removes the package.
#   packaging/package-check.sh deb|rpm

set -euo pipefail
source "$(dirname "$0")/common.sh"

KIND="${1:-}"
TARGET="${TARGET:-x86_64-unknown-linux-musl}"
case "$TARGET" in
x86_64-unknown-linux-musl)  ARCH=x86_64; DEB_ARCH=amd64 ;;
aarch64-unknown-linux-musl) ARCH=aarch64; DEB_ARCH=arm64 ;;
*) die "no Linux architecture name known for $TARGET" ;;
esac
PORT=18080
FAILED=0

verdict() {
    if eval "$1"; then printf '  ok    %s\n' "$2"; else printf '  FAIL  %s\n' "$2"; FAILED=1; fi
}

http() {
    local method="$1" path="$2" body="${3:-}"
    exec 3<>"/dev/tcp/127.0.0.1/$PORT"
    printf '%s %s HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: %s\r\n\r\n%s' \
        "$method" "$path" "${#body}" "$body" >&3
    cat <&3
    exec 3<&-
}

case "$KIND" in
deb)
    package="$RELEASE/${PROGRAM}_${VERSION}.debian.${DEB_ARCH}.deb"
    [ -f "$package" ] || die "$package is missing; run 'packaging/package.sh deb' first"
    say "installing $package"
    apt-get update -qq
    apt-get install -y -qq "$package"
    ;;
rpm)
    package="$RELEASE/${PROGRAM}_${VERSION}.el.${ARCH}.rpm"
    [ -f "$package" ] || die "$package is missing; run 'packaging/package.sh rpm' first"
    say "installing $package"
    dnf install -y -q "$package"
    ;;
*) echo "usage: packaging/package-check.sh deb|rpm" >&2; exit 2 ;;
esac

say "what the installation made"
verdict '[ -x /usr/bin/home-portal ]' "/usr/bin/home-portal is there"
verdict '[ -f /usr/share/home-portal/web/en/index.html ]' "the interface is in /usr/share/home-portal/web"
verdict 'getent passwd home-portal >/dev/null' "the user home-portal exists"
verdict '[ "$(stat -c %U:%a /etc/home-portal)" = home-portal:750 ]' "/etc/home-portal is home-portal's, 0750"
verdict '[ "$(stat -c %U:%a /etc/home-portal/home-portal.toml)" = home-portal:600 ]' "the configuration is home-portal's, 0600"
verdict '[ "$(stat -c %U:%a /etc/home-portal/initial-password)" = home-portal:600 ]' "initial-password is home-portal's, 0600"
verdict '[ "$(stat -c %U:%a /var/lib/home-portal/scripts)" = root:755 ]' "the scripts directory is root's, 0755"
verdict 'grep -qx "directory = \"/var/lib/home-portal\"" /etc/home-portal/home-portal.toml' "the data goes to /var/lib/home-portal"
verdict 'grep -qx "name = \"admin\"" /etc/home-portal/home-portal.toml' "the configuration holds admin"

say "the portal runs as home-portal and admin signs in"
password="$(cat /etc/home-portal/initial-password)"
setpriv --reuid=home-portal --regid=home-portal --init-groups \
    env HOME_PORTAL_CONFIG=/etc/home-portal/home-portal.toml HOME_PORTAL_ADDRESS="127.0.0.1:$PORT" \
    /usr/bin/home-portal > /tmp/portal.log 2>&1 &
portal=$!
for _ in $(seq 1 50); do
    (exec 3<>"/dev/tcp/127.0.0.1/$PORT") 2>/dev/null && break
    sleep 0.2
done
verdict 'http GET /health | head -1 | grep -q " 200 "' "GET /health answers 200"
verdict 'http GET / | grep -qi "^content-type: text/html"' "GET / answers the interface from /usr/share/home-portal/web"
verdict 'http POST /api/session "{\"name\":\"admin\",\"password\":\"$password\"}" | head -1 | grep -q " 20[04] "' \
    "admin signs in with the password from initial-password"
verdict 'http POST /api/session "{\"name\":\"admin\",\"password\":\"wrong\"}" | head -1 | grep -q " 401 "' \
    "a wrong password is refused"
verdict '[ -d /var/lib/home-portal/icons ]' "the portal writes its data into /var/lib/home-portal"
kill "$portal"; wait "$portal" 2>/dev/null || true
[ "$FAILED" -eq 0 ] || cat /tmp/portal.log

say "an upgrade in place keeps the configuration"
before="$(sha256sum /etc/home-portal/home-portal.toml)"
case "$KIND" in
deb) dpkg -i "$package" >/dev/null ;;
rpm) rpm -Uvh --replacepkgs "$package" >/dev/null ;;
esac
verdict '[ "$(sha256sum /etc/home-portal/home-portal.toml)" = "$before" ]' "the configuration is untouched"

say "removal"
case "$KIND" in
deb) apt-get remove -y -qq home-portal ;;
rpm) dnf remove -y -q home-portal ;;
esac
verdict '[ ! -e /usr/bin/home-portal ]' "the binary is gone"
verdict '[ -f /etc/home-portal/home-portal.toml ]' "the configuration stays"

[ "$FAILED" -eq 0 ] || die "the $KIND is not what it should be"
say "the $KIND installs, runs and is removed"
