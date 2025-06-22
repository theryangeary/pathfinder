#!/bin/bash

# Git hooks installation script
# This script creates delegating hooks in .git/hooks/ that invoke the checked-in versions
# from the hooks/ directory, ensuring they stay in sync automatically.

set -e

echo "Installing delegating git hooks..."

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

# Create delegating pre-commit hook
if [ -f "hooks/pre-commit" ]; then
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash

# Pre-commit hook: Invoke the checked-in version
exec ./hooks/pre-commit "$@"
EOF
    chmod +x .git/hooks/pre-commit
    chmod +x hooks/pre-commit
    echo "✅ Installed delegating pre-commit hook"
else
    echo "Warning: hooks/pre-commit not found"
fi

# Create delegating pre-push hook
if [ -f "hooks/pre-push" ]; then
    cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash

# Pre-push hook: Invoke the checked-in version
exec ./hooks/pre-push "$@"
EOF
    chmod +x .git/hooks/pre-push
    chmod +x hooks/pre-push
    echo "✅ Installed delegating pre-push hook"
else
    echo "Warning: hooks/pre-push not found"
fi

echo ""
echo "Delegating git hooks installed successfully!"
echo ""
echo "The following hooks are now active:"
echo "- pre-commit: Runs linting and tests before commits"
echo "- pre-push: Runs linting and all tests before pushes"
echo ""
echo "These hooks automatically invoke the checked-in versions in hooks/"
echo "so they will always stay in sync with your repository."
echo ""
echo "To skip hooks temporarily, use:"
echo "  git commit --no-verify"
echo "  git push --no-verify"