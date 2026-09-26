# Build the interface into web/out and the release binary, which serves it from HOME_PORTAL_WEB or a package
[group('build')]
build: web
    {{ CARGO }} build --release -p home-portal
