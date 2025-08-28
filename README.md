# Pali Terminal Client

[![CI](https://github.com/pali-org/terminal/actions/workflows/ci.yml/badge.svg)](https://github.com/pali-org/terminal/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-%23dea584.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/pali-terminal.svg)](https://crates.io/crates/pali-terminal)

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

**Full-featured interactive terminal interface with:**

**Core Features:**
- **Real-time todo management** - View, create, toggle, and delete todos
- **Beautiful interface** - Color-coded priorities, completion status indicators
- **Multiple screens** - Todo list, add form, help, and settings
- **Keyboard navigation** - Vim-like (h/j/k/l) and arrow key support
- **Priority indicators** - Visual ! / !! / !!! for low/medium/high priority
- **Loading states** - Smooth UX with loading overlays during API calls
- **Error handling** - User-friendly error and success messages

**Navigation:**
- `↑/j` - Move up, `↓/k` - Move down
- `n/a` - Add new todo
- `Enter/Space` - Toggle completion status
- `d` - Delete selected todo
- `r` - Refresh todo list
- `h/?` - Show help screen
- `s` - Settings screen
- `q/Esc` - Quit or go back

**Screens:**
- **Todo List** - Main interface with all todos
- **Add Todo** - Form for creating new todos (title, description, priority)
- **Help** - Complete keyboard shortcuts and usage guide
- **Settings** - View current configuration

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
- `http-optimized` - Enables optimized HTTP client with Hickory DNS and Rustls (default)

Build configurations:
```bash
# Full build with all optimizations (default)
cargo build

# CLI only with HTTP optimizations
cargo build --no-default-features --features cli,http-optimized

# TUI only with HTTP optimizations
cargo build --no-default-features --features tui,http-optimized

# Standard HTTP client (no optimizations)
cargo build --no-default-features --features cli

# Both CLI and TUI with standard HTTP
cargo build --no-default-features --features cli,tui
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
cargo test --all-features

# Check code quality
cargo clippy -- -D warnings
```

## API Integration

The client is designed to work seamlessly with Pali servers and supports:

- **Authentication**: `X-API-Key` header
- **Response format**: `{success: bool, data?: T, error?: string}`
- **Error handling**: Descriptive HTTP status codes
- **All endpoints**: Complete CRUD + admin operations

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

[License details to be added]