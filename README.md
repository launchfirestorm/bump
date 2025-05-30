# Bump

A simple command-line tool for semantic versioning management.

```
 ____  __  __  __  __  ____ 
(  _ \(  )(  )(  \/  )(  _ \
 ) _ < )(__)(  )    (  )___/
(____/(______)(_/\/\_)(__)  
```

## Features

- Bump major, minor, or patch version
- Generate C/C++ header files with version definitions

## Installation

```bash
cargo install bump
```

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

## Usage

```bash
# Show version information
bump --version

# Bump the major version
bump --major

# Bump the minor version
bump --minor

# Bump the patch version
bump --patch

# Output a C header file, default is "version.h"
bump --patch --output-file <FILE>
```

## License

MIT License
