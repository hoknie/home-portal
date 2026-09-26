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

# CHANGELOG.md anew from the commits between v* tags
[group('release')]
changelog:
    packaging/changelog.sh rebuild

# The version the next release takes (auto from the commits, or patch/minor/major)
[group('release')]
next-version level='auto':
    @packaging/changelog.sh next {{ level }}

# Bump Cargo.toml unless it is already past the last tag, then write its CHANGELOG.md section
[group('release')]
release level='auto':
    packaging/changelog.sh release {{ level }}
