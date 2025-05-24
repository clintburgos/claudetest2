#!/bin/bash
# Pre-commit hook for the creature simulation project
# Copy this file to .git/hooks/pre-commit and make it executable:
# cp scripts/pre-commit.sh .git/hooks/pre-commit
# chmod +x .git/hooks/pre-commit

set -e

echo "Running pre-commit checks..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Not in project root directory"
    exit 1
fi

# Format check
echo "Checking formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Formatting check failed!"
    echo "Run 'cargo fmt' to fix formatting issues"
    exit 1
fi
echo "✅ Formatting OK"

# Clippy lints
echo "Running clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy found issues!"
    exit 1
fi
echo "✅ Clippy OK"

# Check for TODO/FIXME comments
echo "Checking for TODO comments..."
if grep -r "TODO\|FIXME\|XXX\|HACK" src/ --exclude-dir=target; then
    echo "⚠️  Found TODO comments. Please address or create GitHub issues."
    # This is a warning, not a failure
fi

# Run tests
echo "Running tests..."
if ! cargo test --lib --bins; then
    echo "❌ Tests failed!"
    exit 1
fi
echo "✅ Tests passed"

# Check documentation
echo "Checking documentation..."
if ! cargo doc --no-deps --document-private-items --quiet; then
    echo "❌ Documentation errors!"
    exit 1
fi
echo "✅ Documentation OK"

# Check for large files
echo "Checking file sizes..."
large_files=$(find src/ -type f -size +500k)
if [ ! -z "$large_files" ]; then
    echo "⚠️  Large files detected:"
    echo "$large_files"
    echo "Consider moving large assets outside of src/"
fi

echo "✅ All pre-commit checks passed!"
