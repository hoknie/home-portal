set shell := ["bash", "-c"]

CARGO := env('CARGO', 'cargo')
PNPM := env('PNPM', 'pnpm')
WEB := 'web'

[private]
default:
    @{{ just_executable() }} --list

# Format the Rust code
fmt:
    {{ CARGO }} fmt --all

# Check Rust formatting without writing
fmt-check:
    {{ CARGO }} fmt --all --check

# Lint the Rust code, warnings are errors
clippy:
    {{ CARGO }} clippy --workspace --all-targets -- -D warnings

# Every Rust test in the workspace
test:
    {{ CARGO }} test --workspace

# Install the frontend dependencies exactly as locked
web-install:
    {{ PNPM }} --dir {{ WEB }} install --frozen-lockfile

# Lint the frontend and check its layers
web-lint: web-install
    {{ PNPM }} --dir {{ WEB }} lint

# Typecheck the frontend
web-typecheck: web-install
    {{ PNPM }} --dir {{ WEB }} typecheck

# Every frontend test
web-test: web-install
    {{ PNPM }} --dir {{ WEB }} test

# Build the interface into web/out, which the portal embeds
web: web-install
    {{ PNPM }} --dir {{ WEB }} build

# The whole gate, in order
check: fmt-check clippy test web-lint web-typecheck web-test web

# Build the interface, then the release binary that embeds it
build: web
    {{ CARGO }} build --release -p home-portal

# Write the API samples the frontend schemas are checked against
samples:
    HOME_PORTAL_SAMPLES=write {{ CARGO }} test -p home-portal --test samples

# Run the portal against config/home-portal.toml (HOME_PORTAL_CONFIG changes the path)
run:
    HOME_PORTAL_CONFIG="${HOME_PORTAL_CONFIG:-config/home-portal.toml}" {{ CARGO }} run -p home-portal

# How to work on the interface with live reload
dev:
    @echo "terminal 1: just run"
    @echo "terminal 2: pnpm --dir web dev   (proxies /api to HOME_PORTAL_ORIGIN, default http://127.0.0.1:8080)"

# A static Linux binary and its archive in dist/release; TARGET=aarch64-unknown-linux-musl for arm64
package: web
    TARGET="${TARGET:-x86_64-unknown-linux-musl}" packaging/package.sh all

# A universal macOS binary and its archive in dist/release
package-macos: web
    packaging/package-macos.sh all

# SHA256SUMS over the archives in dist/release
checksums:
    packaging/release.sh checksums
