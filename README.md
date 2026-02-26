# `bump` automatic versioning

> An un-opinionated command-line tool for **SemVer** and **CalVer** management with TOML-based configuration, multi-language code generation, and git-aware version detection. No assumption is made, you `bump` when you want, how you want.

## Why?
I got tired of bespoke scripts and tons of regex parsing that differentiated slightly from repo to repo just to bump versions. So I created `bump` to be _dead simple_ and **without opinion**. Everyone wants to version differently and that's okay, with a sprinkling of convention and a large helping of automation this tool allows you to never have to worry about versions again!

## Features

- **Dual Versioning Schemes**: Choose between **SemVer** (semantic versioning) or **CalVer** (calendar versioning) based on your project needs
- **TOML Configuration**: Declare your version in a file and `bump` will modify it automatically. Define behavior and preference from the same file. Comments are preserved. Create one with `bump init`
- **Easy Build System Integration**: Define in _one place_ and use everywhere with `bump --print`
- **Semantic Versioning**: Full support for `--major`, `--minor`, `--patch`, `--candidate`, and `--release` bumps with configurable promotion strategies
- **Calendar Versioning**: Date-based versions with strftime format patterns (e.g., `v2026.02.25`) and automatic conflict resolution
- **Multi-Language Support**: Generate version files for C, Go, Java, C#, and Python - useful for injecting version strings into binaries
- **Git Integration**: Automatic SHA appending for untagged commits, branch detection, and smart tag conflict handling
- **Flexible Output**: Support for multiple output files from a single `bump.toml`
- **Comment Preservation**: All comments and formatting in your TOML file are preserved across updates


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

Bump uses a TOML configuration file (`bump.toml`) to manage versioning settings. You can choose between **SemVer** or **CalVer** when initializing.

### SemVer Configuration

```toml
#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

[semver.format]
prefix = "v"
delimiter = "."
timestamp = "%Y-%m-%d %H:%M:%S %Z"   # [optional] strftime syntax for build timestamp

# NOTE: This section is modified by the bump command
[semver.version]
major = 0
minor = 0
patch = 0
candidate = 0

# Candidate promotion strategies:  (when creating first candidate)
#  - "major" : increment major, zero minor and patch
#  - "minor" : increment minor, zero patch
#  - "patch" : increment patch
[semver.candidate]
promotion = "minor"
delimiter = "-rc"

# Development suffix strategies:
#  - "git_sha" : append 7 char sha1 of the current commit (default)
#  - "branch"  : append the current git branch name
#  - "full"    : append <branch>_<sha1>
[semver.development]
promotion = "git_sha"
delimiter = "+"
```

### CalVer Configuration

```toml
#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

# format will drive version section below
# - remove optional fields to change format
# - for minor|micro, setting to false is the same as removing
[calver.format]
prefix = ""
delimiter = "."
year = "%Y"        # strftime 4 digit year
month = "%m"       # [optional] strftime zero padded month
day = "%d"         # [optional] strftime zero padded day
minor = false      # [optional] minor version number
micro = false      # [optional] micro version number

# NOTE: This section is modified by the bump command
[calver.version]
year = "2026"
month = "02"
day = "25"

# Conflict resolution when same date matches existing version:
#  - "suffix"    : append numeric suffix (e.g., 2026.02.25-1)
#  - "overwrite" : reuse the same version
# NOTE: suffix is modified by the bump command
[calver.conflict]
resolution = "suffix"
suffix = 0
delimiter = "-"
```

### Configuration Options

#### SemVer Options

- **`[semver.format]`**:
  - `prefix`: Version tag prefix (e.g., "v", "release-", or empty string)
  - `delimiter`: Separator between version components (default: ".")
  - `timestamp`: strftime format string for timestamps in generated files (optional)
- **`[semver.version]`**: Current version numbers (automatically updated by bump commands)
- **`[semver.candidate]`**: 
  - `promotion`: Which version component to bump when promoting candidates ("minor", "major", or "patch")
  - `delimiter`: Separator for candidate versions (default: "-rc")
- **`[semver.development]`**: 
  - `promotion`: Strategy for development versions ("git_sha", "branch", or "full")
  - `delimiter`: Separator for development versions (default: "+")

#### CalVer Options

- **`[calver.format]`**:
  - `prefix`: Version tag prefix (e.g., "v", "release-", or empty string)
  - `delimiter`: Separator between date components (default: ".")
  - `year`: strftime format for year (e.g., "%Y" for 4-digit year)
  - `month`: strftime format for month (optional, e.g., "%m" for zero-padded month)
  - `day`: strftime format for day (optional, e.g., "%d" for zero-padded day)
  - `minor`: Include minor version number (optional, boolean)
  - `micro`: Include micro version number (optional, boolean)
- **`[calver.version]`**: Current date components (automatically updated by bump --calendar command)
- **`[calver.conflict]`**: 
  - `resolution`: How to handle same-day version conflicts ("suffix" or "overwrite")
  - `suffix`: Current suffix number (automatically incremented when conflicts detected)
  - `delimiter`: Separator for suffix (default: "-")

**Note**: All comments in your `bump.toml` are preserved when the file is updated!

## Commands

### Basic Commands

```bash
# Initialize a new bump.toml file (defaults to SemVer)
bump init [bump.toml]

# Initialize with CalVer instead
bump init --calver [bump.toml]

# Print current version
bump --print [bump.toml]      # Full version with suffixes
bump --print-base [bump.toml] # Base version only
```

### SemVer Commands

```bash
# Bump version numbers (updates bump.toml)
bump --major     # 1.0.0 -> 2.0.0
bump --minor     # 1.0.0 -> 1.1.0  
bump --patch     # 1.0.0 -> 1.0.1
bump --candidate # 1.0.0 -> 1.1.0-rc1 (or increment rc if already candidate)
bump --release   # 1.1.0-rc1 -> 1.1.0 (promote candidate to release)
```

### CalVer Commands

```bash
# CalVer versions are automatically generated from the current date
bump --calendar  # Updates to current date (e.g., 2026.02.25)

# If same date already in version section:
# - With "suffix" resolution: 2026.02.25-1, 2026.02.25-2, etc.
# - With "overwrite" resolution: reuses 2026.02.25
```

### Code Generation

```bash
# Generate version files for different languages
bump gen --lang=c version.h
bump gen --lang=go version.go
bump gen --lang=java Version.java
bump gen --lang=csharp Version.cs
bump gen --lang=python version.py

# Generate multiple files at once
bump gen --lang=c -f bump.toml version.h include/version.h

# SemVer generates: VERSION, VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH, etc.
# CalVer generates: VERSION_STRING only (simplified for date-based versions)
```

### Git Integration

```bash
# Create a git tag for the current version
bump tag

# Commit and tag in one step
bump --minor
git commit -am "Bump to $(bump -p)"
bump tag
```

## Workflow Examples

### SemVer Release Pipeline

For traditional semantic versioning with candidates:

```bash
# Initialize SemVer project
bump init

# Develop with automatic git SHA suffixes
bump gen --lang=c version.h  # Generates: 0.0.0+a1b2c3d (untagged)

# Create a release candidate
bump --candidate              # 0.0.0 -> 0.1.0-rc1
git commit -am "Bump to $(bump -p)"
bump tag

# Test and iterate
bump --candidate              # 0.1.0-rc1 -> 0.1.0-rc2
git commit -am "Bump to $(bump -p)"
bump tag

# Promote to release
bump --release                # 0.1.0-rc2 -> 0.1.0
git commit -am "Release $(bump -p)"
bump tag

# Generate final version files (detects tag, no SHA)
bump gen --lang=c version.h
bump gen --lang=go version.go
# Build and deploy...
```

### CalVer Release Pipeline

For date-based versioning ideal for continuous deployment:

```bash
# Initialize CalVer project
bump init --calver

# Each day gets a new version automatically
bump --calendar               # Generates: 2026.02.25
git commit -am "Release $(bump -p)"
bump tag
bump gen --lang=python version.py
# Build and deploy...

# Multiple releases same day? Automatic suffix handling:
bump --calendar               # 2026.02.25-1
git commit -am "Hotfix $(bump -p)"
bump tag
bump gen --lang=python version.py
# Build and deploy...
```

**PRO TIP**: Add generated version files to `.gitignore` to avoid "behind by one" issues


## Version Scheme Comparison

| Feature | SemVer | CalVer |
|---------|--------|--------|
| **Format** | major.minor.patch | Customizable date format |
| **Example** | v1.2.3, v1.2.0-rc1 | 2026.02.25, 2026.02.25-1 |
| **Best For** | Libraries, APIs, traditional releases | SaaS, continuous deployment |
| **Bumping** | --major, --minor, --patch | --calendar |
| **Candidates** | Yes (--candidate, --release) | No (use suffix for same-day releases) |
| **Generated Code** | All version components | VERSION_STRING only |
| **Conflict Resolution** | N/A | Automatic suffix increment or overwrite |

## [MIT License](./LICENSE)
