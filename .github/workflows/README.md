# GitHub Actions Auto-Merge Workflow

This directory contains GitHub Actions workflows for automated testing and merging of pull requests.

## Workflows

### CI Workflow (`ci.yml`)
- **Triggers**: Push to master, pull requests to master, manual dispatch
- **Jobs**:
  - `test`: Runs tests on Ubuntu with multiple Rust versions (stable, beta, nightly, 1.78.0)
  - `test-windows`: Runs tests on Windows with stable Rust
  - `clippy_check`: Runs Clippy linting
  - `ci-success`: Summary job that ensures all CI jobs pass
- **Tests Include**:
  - Rust compilation (`cargo build`)
  - Unit tests (`cargo test --all`)
  - Code formatting check (`cargo fmt -- --check`)
  - Integration tests (`tests/run_all.sh`)

### Auto-Merge Workflow (`auto-merge.yml`)
- **Triggers**: 
  - Pull request events (opened, synchronized, reopened, labeled)
  - Successful completion of CI workflow
- **Behavior**: Automatically merges pull requests when all tests pass

## How to Use Auto-Merge

### Option 1: Label-Based Auto-Merge
1. Create a pull request
2. Add the `auto-merge` label to the PR
3. The workflow will automatically merge the PR once all CI checks pass

### Option 2: Automatic Trigger (All PRs)
- The workflow automatically triggers when CI completes successfully
- PRs will be merged automatically unless they have the `no-merge` label

### Labels
- **`auto-merge`**: Marks a PR for automatic merging when tests pass
- **`no-merge`**: Prevents automatic merging even if tests pass

## Branch Protection (Recommended)
To ensure the auto-merge workflow works properly, configure branch protection rules for the `master` branch:

1. Go to Settings → Branches in your GitHub repository
2. Add a branch protection rule for `master`
3. Enable "Require status checks to pass before merging"
4. Select the following required status checks:
   - `ci-success`
   - `test`
   - `test-windows` 
   - `Clippy`

## Features
- ✅ Waits for all CI jobs to complete before merging
- ✅ Uses squash merge by default
- ✅ Automatically deletes merged branches
- ✅ Adds comments when PRs are auto-merged
- ✅ Supports both label-based and automatic triggers
- ✅ Respects `no-merge` label to prevent unwanted merges
- ✅ Comprehensive error handling and status checking

## Security
- Uses `GITHUB_TOKEN` with appropriate permissions
- Only merges non-draft pull requests
- Requires all status checks to pass
- Respects branch protection rules
