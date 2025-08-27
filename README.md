# Pali Terminal Client

Production-ready terminal interfaces for the Pali todo management system.

## Overview

This crate provides both command-line and terminal user interface tools for interacting with Pali servers:

- **`pacli`** - Command-line interface for automation and scripting
- **`patui`** - Terminal user interface for interactive usage (placeholder)

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/pali-org/terminal.git
cd terminal

# Build both binaries
cargo build --release
```

### Initialize Your Setup

```bash
# One-command setup: configure endpoint and get your admin key
./target/release/pacli init https://your-server.workers.dev

# Start managing todos immediately
./target/release/pacli add "My first todo"
./target/release/pacli list
```

## Features

### CLI (`pacli`)

**Todo Management:**
- `pacli add <title>` - Create new todos
- `pacli list` - List all todos (with filtering options)
- `pacli get <id>` - Get specific todo details
- `pacli update <id>` - Update existing todos
- `pacli delete <id>` - Delete todos
- `pacli toggle <id>` - Toggle completion status
- `pacli complete <id>` - Mark as complete
- `pacli search <query>` - Search todos

**Configuration:**
- `pacli config endpoint <url>` - Set API endpoint
- `pacli config key <key>` - Set API key
- `pacli config show` - Show current configuration

**Admin Operations:**
- `pacli admin rotate-key` - Rotate admin API key
- `pacli admin generate-key` - Generate new API keys
- `pacli admin list-keys` - List all API keys
- `pacli admin revoke-key <id>` - Revoke API keys
- `pacli admin reinitialize` - Emergency server reset

### TUI (`patui`)

Interactive terminal interface (currently placeholder - CLI is fully functional).

## Architecture

### Multi-Binary Design

```
src/
├── bin/
│   ├── pacli.rs     # CLI binary
│   └── patui.rs     # TUI binary
├── cli/             # CLI-specific code
├── tui/             # TUI-specific code  
├── api.rs           # HTTP client
├── config.rs        # Configuration management
└── lib.rs           # Library root with feature gates
```

### Feature Gates

- `cli` - Enables CLI functionality (default)
- `tui` - Enables TUI functionality (default)

Build only what you need:
```bash
# CLI only
cargo build --no-default-features --features cli

# TUI only  
cargo build --no-default-features --features tui
```

## Configuration

Configuration is stored at:
- **Linux/macOS**: `~/.config/pali/config.json`
- **Windows**: `%APPDATA%/pali/config.json`

```json
{
  "api_endpoint": "https://your-server.workers.dev",
  "api_key": "your-api-key-here"
}
```

⚠️ **Security Notice**: API keys are stored in plain text. The CLI will warn you about this and show the config file location.

## Development

### Requirements

- Rust 1.70+ 
- Cargo

### Building

```bash
# Development build
cargo build

# Release build  
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy -- -W clippy::pedantic
```

### Code Quality

This codebase maintains exemplary standards:
- **Zero clippy warnings** (even with `--pedantic`)
- **Comprehensive error documentation** for all functions
- **17 unit tests** with 100% pass rate
- **Modern Rust idioms** throughout

## API Integration

The client is designed to work seamlessly with Pali servers and supports:

- **Authentication**: `X-API-Key` header
- **Response format**: `{success: bool, data?: T, error?: string}`
- **Error handling**: Descriptive HTTP status codes
- **All endpoints**: Complete CRUD + admin operations

## Contributing

This is part of the Pali organization ecosystem:
- **Terminal Client** (this repo) - CLI/TUI interfaces
- **Server** - Cloudflare Workers API
- **Types** - Shared type definitions

See coordination files:
- `CLAUDE.md` - Overall project coordination
- `Claude.1.md` - Terminal client progress
- `Claude.2.md` - Server progress

## License

[License details to be added]