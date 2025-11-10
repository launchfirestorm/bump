# `bump` automatic versioning

> An un-opinionated command-line tool for semantic versioning management with TOML-based configuration, multi-language code generation, and git-aware version detection. No assumption is made, you `bump` when you want, how you want.

## Why?
I got tired of bespoke scripts and tons of regex parsing that differentiated slightly from repo to repo just to bump versions. So I created `bump` to be _dead simple_ and **without opinion**. Everyone wants to version differently and that's okay, with a sprinkling of convention and a large helping of automation this tool allows you to never have to worry about versions again!

## Features

- **TOML Configuration**: Declare your version in a file and `bump` will modify it automatically. Define behavior and preference from the same file. Comments are preserved. Create one with `bump init`
- **Easy Build system integration**: Define in _one place_ and use everywhere with `bump --print`
- **Semantic Versioning**: No surprises, no assumptions, `--major`, `--minor`, `--patch` do exactly what you think
- **Multi-Language Support**: Generate version files for C, Go, Java, and C#, useful for injecting version strings into binaries
- **Git Integration**: Automatic SHA appending for untagged commits
- **Flexible Output**: Support for multiple output files from a single `bump.toml`
- **Promotion Strategies**: Configurable promotion strategies for both Development and Candidates


## Installation

### Quick Install (Cross Platform)

Install bump with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/launchfirestorm/bump/main/install.sh | bash
```

Or download and run manually:

```bash
wget https://raw.githubusercontent.com/launchfirestorm/bump/main/install.sh
chmod +x install.sh
./install.sh
```

This will:
- Detect your OS (Linux, macOS, Windows) and architecture (x86_64/arm64)
- Download the appropriate binary from the latest GitHub release
- Install to `/usr/local/bin` (or appropriate system directory)
- Verify the installation
- **No authentication required** - downloads from public releases

### Manual Installation

Go to the [releases page](https://github.com/launchfirestorm/bump/releases) and grab the latest binary for your platform.

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

## Configuration

Bump uses a TOML configuration file (`bump.toml`) to manage versioning settings:

```toml
#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

prefix = "v"
timestamp = "%Y-%m-%d %H:%M:%S"   # strftime syntax

# NOTE: This section is modified by the bump command
[version]
major = 0
minor = 0
patch = 0
candidate = 0

[candidate]
promotion = "minor"  # ["minor", "major", "patch"]
delimiter = "-rc"

# promotion strategies:
#  - git_sha ( 7 char sha1 of the current commit )
#  - branch ( append branch name )
#  - full ( <branch>_<sha1> )
[development]
promotion = "git_sha"
delimiter = "+"
```

### Configuration Options

- **`prefix`**: Version tag prefix (e.g., "v", "release-", or empty string)
- **`timestamp`**: a strftime syntax string that will be added to output files (see `bump gen`)
- **`[version]`**: Current version numbers (automatically updated by bump commands)
- **`[candidate]`**: 
  - `promotion`: Which version component to bump when creating candidates ("minor", "major", "patch")
  - `delimiter`: Separator for candidate versions (default: "-rc")
- **`[development]`**: 
  - `promotion`: Strategy for development versions ("git_sha", "branch", "full")
  - `delimiter`: Separator for development versions (default: "+")

**Comment Preservation**: All comments in your `bump.toml` are preserved when the file is updated!

## Commands

### Versioning Commands

```bash
# Initialize a new bump.toml file
bump init [bump.toml]

# Bump version numbers (updates bump.toml)
bump --major     # 1.0.0 -> 2.0.0
bump --minor     # 1.0.0 -> 1.1.0  
bump --patch     # 1.0.0 -> 1.0.1
bump --candidate # 1.0.0 -> 1.1.0-rc1 (or increment rc if already candidate)
bump --release   # 1.1.0-rc1 -> 1.1.0

# Print current version
bump --print [bump.toml]      # With candidate suffix if applicable
bump --print-base [bump.toml] # Base version only (no candidate suffix)
```

## Workflow

### Release Pipeline
For tagged releases (point releases and candidates):

```bash
# Version your changes
bump --major|--minor|--patch|--candidate|--release  # Updates bump.toml
git commit -m "Bump to version $(bump -p)"
bump tag

# PRO TIP: Add these generated files to .gitignore, that way you don't fall into the "behind by one" trap

# Generate version files (detects git tag, no SHA appended)
bump gen --lang=c -f bump.toml version.h include/version.h
bump gen --lang=go version.go
bump gen --lang=java Version.java
bump gen --lang=csharp Version.cs

# ..build code 
```


## Key Changes in v5.0.0

- **New**: TOML-based configuration format (`bump.toml`)
- **Enhanced**: Comment preservation when updating configuration files
- **Improved**: Cross-platform installation script (no authentication required)
- **Added**: Configurable candidate and development promotion strategies
- **Added**: `tag` subcommand for creating git tags
- **Breaking**: Configuration file format changed from flat file to TOML
- **Enhanced**: Better error handling and user experience

## [MIT License](./LICENSE)
