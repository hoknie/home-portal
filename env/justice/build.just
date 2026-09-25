# Build the interface, then the release binary that embeds it
[group('build')]
build: web
    {{ CARGO }} build --release -p home-portal
