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

# Write the API samples the frontend schemas are checked against
[group('quality')]
samples:
    HOME_PORTAL_SAMPLES=write {{ CARGO }} test -p home-portal --test samples

# The whole gate, in CI's order
[group('quality')]
check: fmt-check clippy test web-lint web-typecheck web-test web
