# Git Hooks

This directory contains git hooks that automatically ensure code quality.

## Available Hooks

- **pre-commit**: Runs frontend tests and backend tests (including database tests) before allowing commits
- **pre-push**: Runs all tests before allowing pushes to remote repository

## Installation

To install the git hooks for your local repository:

```bash
./hooks/install.sh
```

## Manual Installation

If you prefer to install manually:

```bash
# Copy hooks to .git/hooks/
cp hooks/pre-commit .git/hooks/pre-commit
cp hooks/pre-push .git/hooks/pre-push

# Make them executable
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/pre-push
```
