# Bump

A simple command-line tool for semantic versioning management with git-aware header generation.

```
 ____  __  __  __  __  ____ 
(  _ \(  )(  )(  \/  )(  _ \
 ) _ < )(__)(  )    (  )___/
(____/(______)(_/\/\_)(__)  
```

## Features

- Bump major, minor, patch, candidate, or release versions
- Generate C/C++ header files with git-aware version detection
- Automatic SHA appending for untagged commits
- Support for multiple header file generation from a single bumpfile
- Git repository integration for smart version detection

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

## Semantic Versioning

1. **MAJOR** = API change (similar to semver spec)
2. **MINOR** = cadence release from 6 weeks
3. **PATCH** = merged commit onto main
4. **CANDIDATE** = manually driven by start of release sprint
5. **RELEASE** = coordinator deems worthy, which drops -rc#

## Workflow

### Release Pipeline
For tagged releases (point releases and candidates):

```bash
# Version your changes
bump --major|--minor|--patch|--candidate  # Updates bumpfile
git commit -m "Bump to version X.Y.Z"
git tag vX.Y.Z

# Generate header files (detects git tag, no SHA appended)
bump gen bumpfile version.h include/version.h

# ..build code 
```

### Development Pipeline
For development builds (untagged commits):

```bash
# Generate header files (detects untagged commit, appends SHA)
bump gen bumpfile version.h include/version.h

# ..build code 
```

### Example Version Progression

```bash
# Start with initial version
0.1.0

# Bump to candidate
bump --candidate
0.2.0-rc1
git commit && git tag v$(bump --print-ci)  # outputs 0.2.0-rc1

# Generate header for tagged candidate
bump gen bumpfile version.h
# Generates: #define VERSION_STRING "0.2.0-rc1"

# Continue development (untagged)
git commit -m "Fix bug"
bump gen bumpfile version.h  
# Generates: #define VERSION_STRING "0.2.0-rc1+a1b2c3d"

# Ready for release
bump --release
0.2.0
git commit && git tag v$(bump --print-ci)  # outputs 0.2.0

# Generate header for final release
bump gen bumpfile version.h
# Generates: #define VERSION_STRING "0.2.0"
```

## Commands

### Versioning Commands

```bash
# Initialize a new bumpfile
bump init

# Bump version numbers
bump --major     # 1.0.0 -> 2.0.0
bump --minor     # 1.0.0 -> 1.1.0  
bump --patch     # 1.0.0 -> 1.0.1
bump --candidate # 1.0.0 -> 1.1.0-rc1 (or increment rc if already candidate)
bump --release   # 1.1.0-rc1 -> 1.1.0

# Print current version without new-line
bump --print
```

### Header Generation

```bash
# Generate header files with git-aware versioning
bump gen <bumpfile> <output_file>...

# Examples:
bump gen bumpfile version.h                    # Single header
bump gen bumpfile version.h include/version.h  # Multiple headers
```

The `gen` command automatically:
- Checks if you're in a git repository (fails if not)
- Detects if current commit is tagged using `git describe --exact-match --tags HEAD`
- For **tagged commits**: generates clean version (e.g., "1.2.3" or "1.2.3-rc1")
- For **untagged commits**: appends commit SHA (e.g., "1.2.3+a1b2c3d" or "1.2.3-rc1+a1b2c3d")
- Creates directories as needed (mkdir -p behavior)

## Key Changes in v3.0.0

- **Removed**: `--dev` flag and commit persistence in bumpfile
- **Added**: `gen` subcommand with git integration
- **Enhanced**: Smart version detection based on git tags
- **Improved**: Multiple header file generation support
- **Required**: Git repository for `gen` command (fails if not in git repo)

## License

MIT License
