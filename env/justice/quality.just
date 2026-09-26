# Format the Rust code
[group('quality')]
fmt:
    {{ CARGO }} fmt --all

# Check Rust formatting without writing
[group('quality')]
fmt-check:
    {{ CARGO }} fmt --all --check

# Lint the Rust code, warnings are errors
[group('quality')]
clippy:
    {{ CARGO }} clippy --workspace --all-targets -- -D warnings

# Every Rust test in the workspace
[group('quality')]
test:
    {{ CARGO }} test --workspace

# Every Rust test inside Linux, in Docker, as CI's Ubuntu job runs them; caches live in named volumes
[group('quality')]
test-linux:
    @command -v docker >/dev/null || { echo "test-linux needs Docker" >&2; exit 1; }
    docker run --rm \
        -v "{{ justfile_directory() }}":/src:ro \
        -v home-portal-cargo:/usr/local/cargo/registry \
        -v home-portal-rustup:/usr/local/rustup \
        -v home-portal-target:/target \
        -e CARGO_TARGET_DIR=/target \
        -w /src \
        rust:1 cargo test --workspace --locked

# Every Rust test here, then again inside Linux
[group('quality')]
test-all: test test-linux

# Write the API samples the frontend schemas are checked against
[group('quality')]
samples:
    HOME_PORTAL_SAMPLES=write {{ CARGO }} test -p home-portal --test samples

# The whole gate, in CI's order
[group('quality')]
check: fmt-check clippy test web-lint web-typecheck web-test web
