#!/bin/bash
# Create release tarball for omadoctor
# Usage: ./create-release.sh [version]
# Default version: 0.1.0

set -e

VERSION="${1:-0.1.0}"
ARCH="x86_64"
RELEASE_DIR="omadoctor-v${VERSION}-${ARCH}"
TARBALL="${RELEASE_DIR}.tar.gz"

echo "🔧 Building omadoctor release ${VERSION}..."

# Build release binary
cargo build --release

# Create release directory
echo "📁 Creating release directory..."
rm -rf "${RELEASE_DIR}"
mkdir -p "${RELEASE_DIR}/fixes"

# Copy binary
cp "target/release/omadoctor" "${RELEASE_DIR}/"

# Copy fixes
cp fixes/*.toml "${RELEASE_DIR}/fixes/"

# Create VERSION file
echo "${VERSION}" > "${RELEASE_DIR}/VERSION"

# Copy LICENSE if it exists, otherwise create minimal MIT license
if [ -f "LICENSE" ]; then
    cp LICENSE "${RELEASE_DIR}/"
else
    cat > "${RELEASE_DIR}/LICENSE" << 'EOF'
MIT License

Copyright (c) 2026 ZeroiJ

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
fi

# Create tarball
echo "📦 Creating tarball..."
tar -czf "${TARBALL}" "${RELEASE_DIR}"

# Calculate SHA256 sum
echo ""
echo "🔐 SHA256 checksum:"
sha256sum "${TARBALL}"

echo ""
echo "✅ Release tarball created: ${TARBALL}"
echo ""
echo "Next steps:"
echo "1. Upload ${TARBALL} to GitHub Releases as omadoctor-v${VERSION}-${ARCH}.tar.gz"
echo "2. Update PKGBUILD sha256sums with the checksum above"
echo "3. Test: tar -tzf ${TARBALL}"

# Clean up release directory (keep tarball)
rm -rf "${RELEASE_DIR}"
