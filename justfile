export CARGO_TARGET_DIR := env("CARGO_TARGET_DIR", env("HOME") / ".cache/rs-toolkit-target")

default:
    @just --list

check: fmt-check lint test guards
    @echo "✓ all checks passed"

# Scaffold a new tool that compiles out of the box:  just new-tool pdf-merger
new-tool name:
    #!/usr/bin/env bash
    set -euo pipefail
    dir="tools/{{name}}"
    if [[ -e "$dir" ]]; then
        echo "error: $dir already exists" >&2
        exit 1
    fi
    mkdir -p "$dir/src"
    sed "s/__TOOL_NAME__/{{name}}/g" templates/tool/Cargo.toml.tmpl > "$dir/Cargo.toml"
    cp templates/tool/main.rs.tmpl        "$dir/src/main.rs"
    cp templates/tool/config.rs.tmpl      "$dir/src/config.rs"
    cp templates/tool/config_test.rs.tmpl "$dir/src/config_test.rs"
    cargo build -p "{{name}}"
    echo ""
    echo "✓ created $dir and it compiles"
    echo "  next: edit $dir/src/config.rs, then run  just run {{name}}"

# ---------------------------------------------------------------------------
# Build & run
# ---------------------------------------------------------------------------

# Build the whole workspace
build:
    cargo build --workspace

# Build one tool on its own
build-tool name:
    cargo build -p {{name}}

# Run a tool
run name *args:
    cargo run -p {{name}} -- {{args}}

# Release build of one tool
release name:
    cargo build --release -p {{name}}

# ---------------------------------------------------------------------------
# Individual checks (composed by `check` above, runnable on their own)
# ---------------------------------------------------------------------------

# Run all tests
test:
    cargo test --workspace

# Run all tests, showing output even from passing ones
test-verbose:
    cargo test --workspace -- --nocapture

# Format every crate
fmt:
    cargo fmt --all

# Fail if anything isn't formatted (used by `check`, good for CI)
fmt-check:
    cargo fmt --all --check

# Lint with warnings treated as errors
lint:
    cargo clippy --workspace --all-targets -- -D warnings

guards:
    #!/usr/bin/env bash
    set -euo pipefail
    ok=1
    if grep -rni 'excel' libs/common/ ; then
        echo "✗ libs/common references a tool's domain (matches above) — common must stay generic"
        ok=0
    fi
    if grep -rn 'config\.toml' tools/ --include='*.rs' | grep -v '_test\.rs' ; then
        echo "✗ a tool hard-codes the config path (matches above) — call common::config::load() instead"
        ok=0
    fi
    if [[ "$ok" != 1 ]]; then
        echo "✗ architecture guards failed"
        exit 1
    fi
    echo "✓ architecture guards passed"

# ---------------------------------------------------------------------------
# Housekeeping & worktrees
# ---------------------------------------------------------------------------

clean:
    cargo clean

# Start a branch in its own worktree beside this repo:  just work csv-export
work branch:
    git worktree add "../rs-toolkit-{{branch}}" -b "{{branch}}"
    @echo "now run:  cd ../rs-toolkit-{{branch}}"

work-list:
    git worktree list

# Remove a worktree and delete its branch
work-done branch:
    git worktree remove "../rs-toolkit-{{branch}}"
    git branch -d "{{branch}}"
