# Run tests, then build — fails fast if tests don't pass
build: test
    cargo build

# Just run the tests
test:
    cargo test

# Run tests with output shown even for passing tests
test-verbose:
    cargo test -- --nocapture

# Build in release mode
release:
    cargo build --release

# Format + lint + test, useful before committing
check:
    cargo fmt
    cargo clippy -- -D warnings
    cargo test

# Remove build artifacts
clean:
    cargo clean
