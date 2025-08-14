# Bump

A simple command-line tool for semantic versioning management.

```
 ____  __  __  __  __  ____ 
(  _ \(  )(  )(  \/  )(  _ \
 ) _ < )(__)(  )    (  )___/
(____/(______)(_/\/\_)(__)  
```

## Features

- Bump major, minor, patch, candidate, or development versions
- Generate C/C++ header files with version definitions

## Installation

Go to releases page and grab the latest

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

## Semantic

1. MAJOR = api change (similiar to semver spec)
1. MINOR = cadence release from 6 weeks
1. PATCH = merged commit onto main
1. CANDIDATE = manually driven by start of release sprint
1. RELEASE = coordinator deems worthy, which drops -rc# 
1. DEVELOPMENT = append `+<7 char git sha>` to version to showcase a new build, helps debugging too

## Workflow
```
0.1.1
bump --candidate
0.2.0-rc1 

flight test bad!
bump --dev
0.2.0-rc1+<7 char sha>

flight test bad!
bump --dev
0.2.0-rc1+<7 char sha>

flight test good...
bump --candidate
0.2.0-rc2

===== release coordinater (let's go)
bump --release
0.2.0
```

## License

MIT License
