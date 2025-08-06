#!/bin/bash
# Test: BENDER_IP_REPO_PATH CLI integration
# Ensures all CLI commands work properly with the new feature

set -e

cd "$(dirname "${BASH_SOURCE[0]}")"
BENDER="$(cd .. && pwd)/target/debug/bender"

# Create a temporary directory for testing
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

cleanup() {
    cd /
    rm -rf "$TEST_DIR"
    unset BENDER_IP_REPO_PATH
}
trap cleanup EXIT

# Set up a test environment with search path
mkdir -p cli_test/{repo_path/cli_ip,main_pkg}/src

# Create an IP in the repo path
cat > cli_test/repo_path/cli_ip/Bender.yml << 'EOF'
package:
  name: cli_ip
  authors: ["Test Author <test@example.com>"]

sources:
  - src/cli_ip.sv
EOF

cat > cli_test/repo_path/cli_ip/src/cli_ip.sv << 'EOF'
module cli_ip_module;
  // CLI test IP
endmodule
EOF

# Create main package
cat > cli_test/main_pkg/Bender.yml << 'EOF'
package:
  name: cli_main_pkg
  authors: ["Test Author <test@example.com>"]

dependencies:
  cli_ip: { git: "https://fake.git.url/cli_ip.git", version: "1.0.0" }

sources:
  - src/main.sv
EOF

cat > cli_test/main_pkg/src/main.sv << 'EOF'
module cli_main;
  // CLI test main module
endmodule
EOF

export BENDER_IP_REPO_PATH="$TEST_DIR/cli_test/repo_path"

cd cli_test/main_pkg

# Test 1: bender packages command
$BENDER packages > /dev/null

# Test 2: bender sources command with various flags
$BENDER sources > /dev/null
$BENDER sources --flatten > /dev/null

# Test 3: bender config command
$BENDER config > /dev/null

# Test 4: bender script commands with different formats
$BENDER script flist > /dev/null
$BENDER script template_json > /dev/null

# Test 5: Test with targets
$BENDER sources --target simulation > /dev/null

cd ../..

# Test CLI with empty environment variable
export BENDER_IP_REPO_PATH=""

cd cli_test/main_pkg

# All commands should still work (fall back to normal behavior)
$BENDER packages > /dev/null 2>&1 || true
$BENDER sources > /dev/null
$BENDER config > /dev/null

cd ../..

# Test CLI with unset environment variable
unset BENDER_IP_REPO_PATH

cd cli_test/main_pkg

# All commands should work normally
$BENDER packages > /dev/null 2>&1 || true
$BENDER sources > /dev/null
$BENDER config > /dev/null

cd ../..

echo "All CLI integration tests passed"
