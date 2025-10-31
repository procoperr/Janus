#!/usr/bin/env bash
set -e

echo "Setting up Janus development environment..."

# Check prerequisites
echo ""
echo "Checking prerequisites..."

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust/Cargo not found. Please install from https://rustup.rs/"
    exit 1
fi
echo "SUCCESS: Rust found: $(rustc --version)"

# Check for Bun
if ! command -v bun &> /dev/null; then
    echo "ERROR: Bun not found. Please install from https://bun.sh/"
    exit 1
fi
echo "SUCCESS: Bun found: $(bun --version)"

# Install Rust components
echo ""
echo "Installing Rust components..."
rustup component add rustfmt clippy

# Install dependencies
echo ""
echo "Installing dependencies..."
bun install

# Setup Git hooks
echo ""
echo "Setting up Git hooks..."
bunx husky install
chmod +x .husky/commit-msg
chmod +x .husky/pre-commit

# Verify installation
echo ""
echo "Running verification checks..."

echo ""
echo "Checking formatting..."
cargo fmt --all -- --check || {
    echo "WARNING: Code needs formatting. Running 'cargo fmt'..."
    cargo fmt --all
}

echo ""
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings || {
    echo "WARNING: Clippy found issues. Please fix them manually."
}

echo ""
echo "Building project..."
cargo build

echo ""
echo "Running tests..."
cargo test

echo ""
echo "SUCCESS: Development environment setup complete!"
echo ""
echo "Next steps:"
echo "  - Commit messages will be automatically validated"
echo "  - Code formatting will be checked on commit"
echo "  - Use 'make format' to format code"
echo "  - Use 'make lint' to run clippy"
echo "  - Use 'make test' to run tests"
echo ""
echo "Commit message format: <type>[scope]: <description>"
echo "   Examples:"
echo "     feat: add new feature"
echo "     fix: resolve bug"
echo "     docs: update README"
echo ""
echo "Happy coding!"
