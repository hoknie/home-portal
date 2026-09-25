# Run the portal against config/home-portal.toml (HOME_PORTAL_CONFIG changes the path)
[group('run')]
run:
    HOME_PORTAL_CONFIG="${HOME_PORTAL_CONFIG:-config/home-portal.toml}" {{ CARGO }} run -p home-portal

# How to work on the interface with live reload
[group('run')]
dev:
    @echo "terminal 1: just run"
    @echo "terminal 2: pnpm --dir web dev   (proxies /api to HOME_PORTAL_ORIGIN, default http://127.0.0.1:8080)"
