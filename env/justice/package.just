# A static Linux binary and its archive in dist/release; TARGET=aarch64-unknown-linux-musl for arm64
[group('package')]
package: web
    TARGET="${TARGET:-x86_64-unknown-linux-musl}" packaging/package.sh all

# Both static Linux archives, x86_64 and aarch64
[group('package')]
package-linux: web
    TARGET=x86_64-unknown-linux-musl packaging/package.sh all
    TARGET=aarch64-unknown-linux-musl packaging/package.sh all

# The deb of a built Linux binary, made in debian:12 (Docker); TARGET as for package
[group('package')]
package-deb:
    docker run --rm --volume "$PWD":/work --workdir /work --env TARGET="${TARGET:-x86_64-unknown-linux-musl}" \
        debian:12 ./packaging/package.sh deb

# The rpm of a built Linux binary, made in almalinux:9 (Docker); TARGET as for package
[group('package')]
package-rpm:
    docker run --rm --volume "$PWD":/work --workdir /work --env TARGET="${TARGET:-x86_64-unknown-linux-musl}" \
        almalinux:9 bash -c 'dnf -y -q install rpm-build >/dev/null && ./packaging/package.sh rpm'

# Install the deb or the rpm on a clean system, run it, sign admin in, remove it (Docker)
[group('package')]
package-check kind:
    docker run --rm --volume "$PWD":/work --workdir /work --env TARGET="${TARGET:-x86_64-unknown-linux-musl}" \
        {{ if kind == "deb" { "debian:12" } else { "almalinux:9" } }} ./packaging/package-check.sh {{ kind }}

# A universal macOS binary, its archive and the .pkg, checked, in dist/release
[group('package')]
package-macos: web
    packaging/package-macos.sh all
