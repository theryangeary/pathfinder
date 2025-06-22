#!/bin/bash

# Git hooks installation script
# This script copies the git hooks from the hooks/ directory to .git/hooks/
# and makes them executable.

set -e

echo "Installing git hooks..."

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "Error: Not in a git repository root directory"
    exit 1
fi

# Check if hooks directory exists
if [ ! -d "hooks" ]; then
    echo "Error: hooks/ directory not found"
    exit 1
fi

# Install pre-commit hook
if [ -f "hooks/pre-commit" ]; then
    cp hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "✅ Installed pre-commit hook"
else
    echo "Warning: hooks/pre-commit not found"
fi

# Install pre-push hook
if [ -f "hooks/pre-push" ]; then
    cp hooks/pre-push .git/hooks/pre-push
    chmod +x .git/hooks/pre-push
    echo "✅ Installed pre-push hook"
else
    echo "Warning: hooks/pre-push not found"
fi

echo ""
echo "Git hooks installed successfully!"
echo ""
echo "The following hooks are now active:"
echo "- pre-commit: Runs frontend tests before commits"
echo "- pre-push: Runs all tests (frontend + backend with database) before pushes"
echo ""
echo "To skip hooks temporarily, use:"
echo "  git commit --no-verify"
echo "  git push --no-verify"