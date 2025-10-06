# Display available commands and their descriptions (default target)
default:
    @just --list


## Workspace management

alias clean := cargo-clean

# Clean cargo workspace (cargo clean)
cargo-clean:
    cargo clean


## Code formatting and linting

alias fmt := fmt-rs
alias fmt-check := fmt-rs-check

# Format specific Rust file
fmt-file FILE:
    #!/usr/bin/env bash
    case "{{FILE}}" in
        *.rs) just fmt-rs-file "{{FILE}}" ;;
    esac

# Format Rust code (cargo fmt --all)
fmt-rs:
    cargo +nightly fmt --all

# Check Rust code format (cargo fmt --check)
fmt-rs-check:
    cargo +nightly fmt --all -- --check

# Format specific Rust file (cargo fmt <file>)
fmt-rs-file FILE:
    cargo +nightly fmt -- {{FILE}}


## Check

alias check := check-rs

# Check Rust code (all targets)
check-all: check-rs

# Check Rust code (cargo check --all-targets)
check-rs *EXTRA_FLAGS:
    cargo check --all-targets {{EXTRA_FLAGS}}

# Check specific crate with tests (cargo check -p <crate> --all-targets)
check-crate CRATE *EXTRA_FLAGS:
    cargo check --package {{CRATE}} --all-targets {{EXTRA_FLAGS}}

# Lint Rust code (cargo clippy --all-targets)
clippy *EXTRA_FLAGS:
    cargo clippy --all-targets {{EXTRA_FLAGS}}

# Lint specific crate (cargo clippy -p <crate> --all-targets)
clippy-crate CRATE *EXTRA_FLAGS:
    cargo clippy --package {{CRATE}} --all-targets {{EXTRA_FLAGS}}


## Testing

alias test := test-all
alias test-it-intree := test-integration-intree
alias test-it-public := test-integration-public
alias test-it-intree-cov := test-integration-intree-cov
alias test-it-public-cov := test-integration-public-cov

OUTPUT_LCOV_DIR := "target/lcov"

# Run all tests (unit, doc, and integration)
test-all: test-unit test-doc test-integration-intree test-integration-public

# Run unit tests only
test-unit *EXTRA_FLAGS:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if cargo-nextest is installed
    if ! command -v "cargo-nextest" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-nextest' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-nextest using cargo:"
        >&2 echo "  cargo install --locked cargo-nextest@^0.9"
        >&2 echo "=============================================================="
        exit 1
    fi

    cargo nextest run --config-file nextest.toml --profile unit --no-tests warn --all-features {{EXTRA_FLAGS}}

# Run documentation tests
test-doc *EXTRA_FLAGS:
    cargo test {{EXTRA_FLAGS}} --all-features --doc

# Run integration tests (in-tree)
test-integration-intree *EXTRA_FLAGS:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if cargo-nextest is installed
    if ! command -v "cargo-nextest" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-nextest' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-nextest using cargo:"
        >&2 echo "  cargo install --locked cargo-nextest@^0.9"
        >&2 echo "=============================================================="
        exit 1
    fi

    cargo nextest run --config-file nextest.toml --profile it-intree --no-tests warn --all-features {{EXTRA_FLAGS}}

# Run integration tests (public API)
test-integration-public *EXTRA_FLAGS:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if cargo-nextest is installed
    if ! command -v "cargo-nextest" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-nextest' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-nextest using cargo:"
        >&2 echo "  cargo install --locked cargo-nextest@^0.9"
        >&2 echo "=============================================================="
        exit 1
    fi

    cargo nextest run --config-file nextest.toml --profile it-public --no-tests warn --all-features --test '*' {{EXTRA_FLAGS}}

# Run end-to-end tests (tests package)
test-e2e *EXTRA_FLAGS:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if cargo-nextest is installed
    if ! command -v "cargo-nextest" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-nextest' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-nextest using cargo:"
        >&2 echo "  cargo install --locked cargo-nextest@^0.9"
        >&2 echo "=============================================================="
        exit 1
    fi

    cargo nextest run --config-file nextest.toml --profile e2e --all-features {{EXTRA_FLAGS}}

# Run unit tests with code coverage
test-unit-cov OUTPUT_DIR=OUTPUT_LCOV_DIR:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if cargo-llvm-cov and cargo-nextest are installed
    if ! command -v "cargo-llvm-cov" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-llvm-cov' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-llvm-cov using cargo:"
        >&2 echo "  cargo install cargo-llvm-cov"
        >&2 echo "=============================================================="
        exit 1
    fi

    if ! command -v "cargo-nextest" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-nextest' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-nextest using cargo:"
        >&2 echo "  cargo install --locked cargo-nextest@^0.9"
        >&2 echo "=============================================================="
        exit 1
    fi

    # Ensure output directory exists
    mkdir -p {{OUTPUT_DIR}}

    echo "Running unit tests with coverage..."
    cargo llvm-cov nextest --config-file nextest.toml --profile unit --no-tests warn --all-features --lcov --output-path {{OUTPUT_DIR}}/unit-lcov.info

    echo "✅ Coverage report generated: {{OUTPUT_DIR}}/unit-lcov.info"

# Run integration tests (in-tree) with code coverage
test-integration-intree-cov OUTPUT_DIR=OUTPUT_LCOV_DIR:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if cargo-llvm-cov and cargo-nextest are installed
    if ! command -v "cargo-llvm-cov" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-llvm-cov' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-llvm-cov using cargo:"
        >&2 echo "  cargo install cargo-llvm-cov"
        >&2 echo "=============================================================="
        exit 1
    fi

    if ! command -v "cargo-nextest" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-nextest' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-nextest using cargo:"
        >&2 echo "  cargo install --locked cargo-nextest@^0.9"
        >&2 echo "=============================================================="
        exit 1
    fi

    # Ensure output directory exists
    mkdir -p {{OUTPUT_DIR}}

    echo "Running integration tests (in-tree) with coverage..."
    cargo llvm-cov nextest --config-file nextest.toml --profile it-intree --no-tests warn --all-features --lcov --output-path {{OUTPUT_DIR}}/it-intree-lcov.info

    echo "✅ Coverage report generated: {{OUTPUT_DIR}}/it-intree-lcov.info"

# Run integration tests (public API) with code coverage
test-integration-public-cov OUTPUT_DIR=OUTPUT_LCOV_DIR:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if cargo-llvm-cov and cargo-nextest are installed
    if ! command -v "cargo-llvm-cov" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-llvm-cov' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-llvm-cov using cargo:"
        >&2 echo "  cargo install cargo-llvm-cov"
        >&2 echo "=============================================================="
        exit 1
    fi

    if ! command -v "cargo-nextest" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-nextest' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-nextest using cargo:"
        >&2 echo "  cargo install --locked cargo-nextest@^0.9"
        >&2 echo "=============================================================="
        exit 1
    fi

    # Ensure output directory exists
    mkdir -p {{OUTPUT_DIR}}

    echo "Running integration tests (public API) with coverage..."
    cargo llvm-cov nextest --config-file nextest.toml --profile it-public --no-tests warn --all-features --test '*' --lcov --output-path {{OUTPUT_DIR}}/it-public-lcov.info

    echo "✅ Coverage report generated: {{OUTPUT_DIR}}/it-public-lcov.info"

# Run end-to-end tests (tests package) with code coverage
test-e2e-cov OUTPUT_DIR=OUTPUT_LCOV_DIR:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if cargo-llvm-cov and cargo-nextest are installed
    if ! command -v "cargo-llvm-cov" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-llvm-cov' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-llvm-cov using cargo:"
        >&2 echo "  cargo install cargo-llvm-cov"
        >&2 echo "=============================================================="
        exit 1
    fi

    if ! command -v "cargo-nextest" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'cargo-nextest' not available ❌"
        >&2 echo ""
        >&2 echo "Please install cargo-nextest using cargo:"
        >&2 echo "  cargo install --locked cargo-nextest@^0.9"
        >&2 echo "=============================================================="
        exit 1
    fi

    # Ensure output directory exists
    mkdir -p {{OUTPUT_DIR}}

    echo "Running end-to-end tests with coverage..."
    cargo llvm-cov nextest --config-file nextest.toml --profile e2e --all-features --lcov --output-path {{OUTPUT_DIR}}/e2e-lcov.info

    echo "✅ Coverage report generated: {{OUTPUT_DIR}}/e2e-lcov.info"


## Codegen

alias codegen := gen

# Run all codegen tasks
gen: \
    gen-tests-fetch-opencli-schema

### OpenCLI specification download
SCHEMAS_DIR := "tests/tests/assets"

# Download the OpenCLI specification schema (RUSTFLAGS="--cfg fetch_opencli_schema" cargo build)
gen-tests-fetch-opencli-schema DEST_DIR=SCHEMAS_DIR:
    RUSTFLAGS="--cfg fetch_opencli_schema" cargo build -p tests
    @echo "OpenCLI spec downloaded to {{DEST_DIR}}/opencli.spec.json"


## Misc

PRECOMMIT_CONFIG := ".github/pre-commit-config.yaml"
PRECOMMIT_DEFAULT_HOOKS := "pre-commit pre-push"

# Install Git hooks
install-git-hooks HOOKS=PRECOMMIT_DEFAULT_HOOKS:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if pre-commit is installed
    if ! command -v "pre-commit" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'pre-commit' not available ❌"
        >&2 echo ""
        >&2 echo "Please install pre-commit using your preferred package manager"
        >&2 echo "  pip install pre-commit"
        >&2 echo "  pacman -S pre-commit"
        >&2 echo "  apt-get install pre-commit"
        >&2 echo "  brew install pre-commit"
        >&2 echo "=============================================================="
        exit 1
    fi

    # Install all Git hooks (see PRECOMMIT_HOOKS for default hooks)
    pre-commit install --config {{PRECOMMIT_CONFIG}} {{replace_regex(HOOKS, "\\s*([a-z-]+)\\s*", "--hook-type $1 ")}}

# Remove Git hooks
remove-git-hooks HOOKS=PRECOMMIT_DEFAULT_HOOKS:
    #!/usr/bin/env bash
    set -e # Exit on error

    # Check if pre-commit is installed
    if ! command -v "pre-commit" &> /dev/null; then
        >&2 echo "=============================================================="
        >&2 echo "Required command 'pre-commit' not available ❌"
        >&2 echo ""
        >&2 echo "Please install pre-commit using your preferred package manager"
        >&2 echo "  pip install pre-commit"
        >&2 echo "  pacman -S pre-commit"
        >&2 echo "  apt-get install pre-commit"
        >&2 echo "  brew install pre-commit"
        >&2 echo "=============================================================="
        exit 1
    fi

    # Remove all Git hooks (see PRECOMMIT_HOOKS for default hooks)
    pre-commit uninstall --config {{PRECOMMIT_CONFIG}} {{replace_regex(HOOKS, "\\s*([a-z-]+)\\s*", "--hook-type $1 ")}}
