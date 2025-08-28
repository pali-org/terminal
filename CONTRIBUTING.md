# Contributing to Pali Terminal

## Development Setup

```bash
# Clone the repository
git clone https://github.com/pali-org/terminal.git
cd terminal

# Build the project
cargo build --all-features

# Run tests
cargo test --all-features
```

## Before Submitting PRs

Run the local CI checks to ensure your code meets quality standards:

```bash
./scripts/ci-check.sh
```

This runs the same checks as our CI pipeline:
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- All tests
- Release builds

## Code Standards

- **Zero warnings**: All clippy warnings must be fixed
- **Formatted code**: Use `cargo fmt` before committing
- **Tests**: Add tests for new functionality
- **Documentation**: Document public APIs

## Project Structure

Part of the Pali ecosystem:
- **Terminal Client** (this repo) - CLI/TUI interfaces
- **[Server](https://github.com/pali-org/server)** - Cloudflare Workers API
- **[Types](https://github.com/pali-org/types)** - Shared type definitions

## Questions?

Check the coordination docs or open an issue for discussion.