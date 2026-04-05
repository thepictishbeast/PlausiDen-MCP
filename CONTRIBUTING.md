# Contributing to PlausiDen MCP Server

## Development Setup

```bash
# Clone
git clone https://github.com/redcaptian1917/plausiden-mcp.git
cd plausiden-mcp

# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

## Running Tests

```bash
# All tests
cargo test

# Verbose output
cargo test -- --nocapture

# Specific test
cargo test test_inject_requires_acknowledgment
```

## Code Style

- `rustfmt` for formatting
- `clippy` with `-D warnings` — no warnings allowed
- `thiserror` for error types
- Never `unwrap()` in library code
- Doc comments (`///`) on all public items

## Submitting Changes

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Write tests for new functionality
4. Ensure `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` all pass
5. Submit a pull request

## Issue Labels

- `good first issue` — suitable for new contributors
- `help wanted` — needs community input
- `security` — security-related changes

## Code of Conduct

This project follows the [Contributor Covenant](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).
