# `bump` automatic versioning

> An un-opinionated command-line tool for **SemVer** and **CalVer** management with TOML-based configuration, multi-language code generation, and git-aware version detection. No assumption is made, you `bump` when you want, how you want.

## Why?
I got tired of bespoke scripts and tons of regex parsing that differentiated slightly from repo to repo just to bump versions. So I created `bump` to be _dead simple_ and **without opinion**. Everyone wants to version differently and that's okay, with a sprinkling of convention and a large helping of automation this tool allows you to never have to worry about versions again!


## Installation

```bash
curl -sSL https://raw.githubusercontent.com/launchfirestorm/bump/main/install.sh | bash
```


## Quick Start

### Initialize a Project

```bash
# Initialize with SemVer (default)
bump init

# Or initialize with CalVer
bump init --calver
```

This creates a **"BUMPFILE"** defaulted to `bump.toml` in your current directory with sensible defaults. You can rename this to how you want.

## Commands

### Reporting Commands

> Often times is getting the version _in other tools_ is the problem. 

```bash
# Print current version
bump --print [BUMPFILE]                    # Full version with suffixes
bump --print-base [BUMPFILE]               # Base version only (no prefix or candidate suffix)
bump --print-with-timestamp [BUMPFILE]     # Version with build timestamp
```

### PRO TIP: you can inject bump _everywhere_
```bash
sed -i "s|REPLACE_ME|$(bump --print)|g" somefile
```

```cmake
# CMakeLists.txt
execute_process(
  COMMAND bump --print-base
  WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}/
  OUTPUT_VARIABLE VERSION)
project("your-app" VERSION ${VERSION} LANGUAGES CXX C)
```


### SemVer Commands

```bash
# Bump version numbers (updates BUMPFILE)
bump --major     # 1.0.0 -> 2.0.0
bump --minor     # 1.0.0 -> 1.1.0  
bump --patch     # 1.0.0 -> 1.0.1
bump --candidate # 1.0.0 -> 1.1.0-rc1 (or increment rc if already candidate)
bump --release   # 1.1.0-rc1 -> 1.1.0 (promote candidate to release)

bump --prefix "release-" --major [BUMPFILE]  # Uses "release-" instead of configured prefix
```

### CalVer Commands

```bash
# CalVer versions are automatically generated from the current date
bump --calendar [BUMPFILE]  # Updates to current date (e.g., 2026.02.25)

# Same-day bumps automatically increment revision:
# First:  2026.02.25    (revision = 0, not shown)
# Second: 2026.02.25-1  (revision = 1)
# Third:  2026.02.25-2  (revision = 2)
# Next day, revision resets to 0
```

### Code Generation

```bash
# Generate version files for different languages
bump gen --lang c --output version.h [BUMPFILE]
bump gen --lang go --output version.go [BUMPFILE]
bump gen --lang java --output Version.java [BUMPFILE]
bump gen --lang csharp --output Version.cs [BUMPFILE]
bump gen --lang python --output version.py [BUMPFILE]

# Generate multiple files at once
bump gen --lang c --output version.h --output include/version.h [BUMPFILE]

# Use custom bumpfile
bump gen --lang c --output version.h custom.toml

# SemVer generates: VERSION, VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH, etc.
# CalVer generates: VERSION_STRING only (simplified for date-based versions)
```

### Git Integration

```bash
# Create a git tag for the current version
bump tag [BUMPFILE]

# Create a tag with custom message
bump tag --message "Release v1.2.3 - Critical security fix" [BUMPFILE]
bump tag -m "Custom message" [BUMPFILE]

# Commit and tag in one step
bump --minor
git commit -am "Bump to $(bump -p)"
bump tag
```

### File Updates

```bash
# Update version in known file types (currently supports Cargo.toml)
bump update Cargo.toml [BUMPFILE]

# This reads version from bump.toml and updates it in Cargo.toml
# Use with custom configuration file
bump update Cargo.toml custom.toml
```

## Features

- **Dual Versioning Schemes**: Choose between **SemVer** (semantic versioning) or **CalVer** (calendar versioning) based on your project needs
- **TOML Configuration**: Declare your version in a file and `bump` will modify it automatically. Define behavior and preference from the same file. Comments are preserved. Create one with `bump init`
- **Easy Build System Integration**: Define in _one place_ and use everywhere with `bump --print`, `bump --print-base`, or `bump --print-with-timestamp`
- **Semantic Versioning**: Full support for `--major`, `--minor`, `--patch`, `--candidate`, and `--release` bumps with configurable promotion strategies
- **Calendar Versioning**: Date-based versions with strftime format patterns (e.g., `2026.02.25`) and automatic revision increment for same-day releases via `--calendar`
- **Multi-Language Support**: Generate version files for C, Go, Java, C#, and Python - useful for injecting version strings into binaries
- **Git Integration**: Automatic SHA appending for untagged commits, branch detection, smart tag conflict handling, and tag creation with custom messages
- **File Updates**: Automatically update versions in known file types (e.g., `Cargo.toml`)
- **Flexible Output**: Support for multiple output files from a single `bump.toml`
- **Comment Preservation**: All comments and formatting in your TOML file are preserved across updates


## Documentation

- **[Configuration Reference](docs/CONFIGURATION.md)** - Detailed configuration options for SemVer and CalVer
- **[Contributing Guide](docs/CONTRIBUTING.md)** - Build from source and development instructions
- **[Workflow Guide](docs/WORKFLOW.md)** - Use `bump` to automate you pipelines!

## [MIT License](./LICENSE)
