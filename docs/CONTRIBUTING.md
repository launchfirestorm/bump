# Contributing to Bump

Thank you for your interest in contributing to bump! This document provides information about building and developing the project.

## Build from Source

This project uses the musl toolchain to produce statically linked binaries that are highly portable across Linux distributions.

### Prerequisites

Install the musl target:

```bash
rustup target add x86_64-unknown-linux-musl
```

### Building

The project is configured to automatically build with musl:

```bash
# Clone the repository
git clone https://github.com/launchfirestorm/bump.git
cd bump

# Build the release version
cargo build --release

# The binary will be available at
# target/x86_64-unknown-linux-musl/release/bump
```

The resulting binary is statically linked and can run on virtually any Linux distribution without additional dependencies.

### Development Build

For faster development builds without musl optimization:

```bash
cargo build
```

The debug binary will be available at `target/debug/bump`.

### Running Tests

```bash
cargo test
```

## Project Structure

```
bump/
├── src/
│   ├── main.rs       # Entry point and command routing
│   ├── cli.rs        # Command-line interface definitions (clap)
│   ├── bump.rs       # Core business logic
│   ├── version.rs    # Version struct, TOML parsing, and bumping logic
│   ├── lang.rs       # Code generation for multiple languages
│   ├── update.rs     # File updating (e.g., Cargo.toml)
│   └── tests/        # Test modules
│       ├── mod.rs
│       ├── semver.rs
│       └── calver.rs
├── docs/             # Documentation
├── Cargo.toml        # Rust project configuration
└── bumpfile          # Bump's own version file
```

## Making Changes

1. Create a feature branch
1. Make your changes
1. Run tests to ensure everything works
1. Submit a pull request

## Questions?

Feel free to open an issue on GitHub if you have questions or need help.
