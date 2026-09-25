# The version in Cargo.toml, the one a release tag must name
[group('release')]
version:
    @packaging/release.sh version

# Check that a tag (vX.Y.Z) names the version in Cargo.toml
[group('release')]
check-tag tag:
    packaging/release.sh check-tag {{ tag }}

# SHA256SUMS over the archives in dist/release
[group('release')]
checksums:
    packaging/release.sh checksums
