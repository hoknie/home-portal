# Install the frontend dependencies exactly as locked
[group('web')]
web-install:
    {{ PNPM }} --dir {{ WEB }} install --frozen-lockfile

# Lint the frontend and check its layers
[group('web')]
web-lint: web-install
    {{ PNPM }} --dir {{ WEB }} lint

# Typecheck the frontend
[group('web')]
web-typecheck: web-install
    {{ PNPM }} --dir {{ WEB }} typecheck

# Every frontend test
[group('web')]
web-test: web-install
    {{ PNPM }} --dir {{ WEB }} test

# Build the interface into web/out, which the packages ship beside the binary
[group('web')]
web: web-install
    {{ PNPM }} --dir {{ WEB }} build
