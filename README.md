```
 ____  __  __  __  __  ____ 
(  _ \(  )(  )(  \/  )(  _ \
 ) _ < )(__)(  )    (  )___/
(____/(______)(_/\/\_)(__)  
```
# `bump` automatic versioning

> An un-opinionated command-line tool for **SemVer** and **CalVer** management with TOML-based configuration and multi-language code generation. No assumption is made, you `bump` when you want, how you want.

### TL;DR
- A **regex-less** way to do versioning 😎
- Human/Machine readable `bump.toml`
- Flexible for your needs
- There stop thinking about versioning!


## Why?
I got tired of bespoke scripts and tons of regex parsing that differentiated slightly from repo to repo just to bump versions. So I created `bump` to be _dead simple_ and **without opinion**. Everyone wants to version differently and that's okay, with a sprinkling of convention and a large helping of automation this tool allows you to never have to worry about versions again!


## Installation

**Linux, macOS, or WSL:**

> ensure you have write permissions to `/usr/local/bin/`, if you need elevation then `... | sudo bash`

```bash
curl -fsSL https://raw.githubusercontent.com/launchfirestorm/bump/main/install/get_bump.sh | bash
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/launchfirestorm/bump/main/install/get_bump.ps1 | iex
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
> All print variants write output **without a trailing newline**.

```bash
# Print current version
bump --print [BUMPFILE]                    # Root version (prefix + candidate suffix if present)
bump --print-base [BUMPFILE]               # Base semver only: MAJOR.MINOR.PATCH (SemVer only)
bump --print-full [BUMPFILE]               # Full version with development suffix policy (git_sha/branch/full)
bump --print-with-timestamp [BUMPFILE]     # Full version + configured build timestamp (SemVer only)
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

**PRO TIP**: Add generated version files to `.gitignore` to avoid "behind by one" issues

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
# Create a git tag for the current version, message is conventional commit format
bump tag [BUMPFILE]

# Create a tag with custom message
bump tag --message "Release v1.2.3 - Critical security fix" [BUMPFILE]
bump tag -m "Custom message" [BUMPFILE]
```

### `bump update`

> Currently supports `Cargo.toml` and `pyproject.toml`, send a PR for additional file format conventions!

```bash
bump update Cargo.toml [BUMPFILE]
bump update pyproject.toml [BUMPFILE]
```


## Documentation

- **[Configuration Reference](docs/CONFIGURATION.md)** - Detailed configuration options for SemVer and CalVer
- **[Contributing Guide](docs/CONTRIBUTING.md)** - Build from source and development instructions
- **[Workflow Guide](docs/WORKFLOW.md)** - Use `bump` to automate you pipelines!

## **GitHub Actions:** 

composite action `action.yml` at repo root installs bump for the job’s OS/arch:

```yaml
- uses: launchfirestorm/bump@v6
```

if your token differs from the default `GITHUB_TOKEN`

```yaml
- uses: launchfirestorm/bump@v6
  with:
    token: ${{ secrets.YOUR_TOKEN_HERE }}
```

## [MIT License](./LICENSE)
