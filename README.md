# Bump

A simple command-line tool for semantic versioning management with multi-language code generation and git-aware version detection.

```
 ____  __  __  __  __  ____ 
(  _ \(  )(  )(  \/  )(  _ \
 ) _ < )(__)(  )    (  )___/
(____/(______)(_/\/\_)(__)  
```

## Features

- Bump major, minor, patch, candidate, or release versions
- Generate version files for multiple programming languages (C, Go, Java, C#)
- Automatic SHA appending for untagged commits
- Support for multiple output files from a single bumpfile
- Git repository integration for smart version detection

## Installation

### Quick Install (Linux)

Install bump with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/launchfirestorm/bump/main/install.sh | bash
```

This will:
- Detect your architecture (x86_64 or arm64)
- Download the appropriate binary from the latest release
- Install to `/usr/local/bin` (if writable) or `~/.local/bin`
- Verify the installation

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

# Generate version files (detects git tag, no SHA appended)
bump gen --lang=c bumpfile version.h include/version.h
bump gen --lang=go bumpfile version.go
bump gen --lang=java bumpfile Version.java
bump gen --lang=csharp bumpfile Version.cs

# ..build code 
```

### Development Pipeline
For development builds (untagged commits):

```bash
# Generate version files (detects untagged commit, appends SHA)
bump gen --lang=c bumpfile version.h include/version.h

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

# Generate version file for tagged candidate
bump gen --lang=c bumpfile version.h
# Generates: #define VERSION_STRING "0.2.0-rc1"

# Continue development (untagged)
git commit -m "Fix bug"
bump gen --lang=c bumpfile version.h  
# Generates: #define VERSION_STRING "0.2.0-rc1+a1b2c3d"

# Ready for release
bump --release
0.2.0
git commit && git tag v$(bump --print-ci)  # outputs 0.2.0

# Generate version file for final release
bump gen --lang=c bumpfile version.h
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

### Code Generation

```bash
# Generate version files with git-aware versioning
bump gen --lang=<LANG> <bumpfile> <output_file>...

# Supported languages: c, go, java, csharp

# Examples:
bump gen --lang=c bumpfile version.h                    # C header file
bump gen --lang=c bumpfile version.h include/version.h  # Multiple C headers
bump gen --lang=go bumpfile version.go                  # Go source file
bump gen --lang=java bumpfile Version.java              # Java class file
bump gen --lang=csharp bumpfile Version.cs              # C# class file
```

## Generated Code Examples

### C Header (`version.h`)
```c
#ifndef BUMP_VERSION_H
#define BUMP_VERSION_H

#define VERSION_MAJOR 1
#define VERSION_MINOR 2  
#define VERSION_PATCH 3
#define VERSION_CANDIDATE 0
#define VERSION_STRING "1.2.3"

#endif /* BUMP_VERSION_H */
```

### Go Source (`version.go`)
```go
package version

const (
	MAJOR     = 1
	MINOR     = 2
	PATCH     = 3
	CANDIDATE = 0
	STRING    = "1.2.3"
)
```

### Java Class (`Version.java`)
```java
public class Version {
    public static final int MAJOR = 1;
    public static final int MINOR = 2;
    public static final int PATCH = 3;
    public static final int CANDIDATE = 0;
    public static final String STRING = "1.2.3";
}
```

### C# Class (`Version.cs`)
```csharp
public static class Version
{
    public const int MAJOR = 1;
    public const int MINOR = 2;
    public const int PATCH = 3;
    public const int CANDIDATE = 0;
    public const string STRING = "1.2.3";
}
```

The `gen` command automatically:
- Checks if you're in a git repository (fails if not)
- Detects if current commit is tagged using `git describe --exact-match --tags HEAD`
- For **tagged commits**: generates clean version (e.g., "1.2.3" or "1.2.3-rc1")
- For **untagged commits**: appends commit SHA (e.g., "1.2.3+a1b2c3d" or "1.2.3-rc1+a1b2c3d")
- Creates directories as needed (mkdir -p behavior)
- Generates language-appropriate syntax and conventions

## Key Changes in v3.0.0

- **Removed**: `--dev` flag and commit persistence in bumpfile
- **Added**: `gen` subcommand with git integration
- **Added**: Multi-language support (C, Go, Java, C#)
- **Enhanced**: Smart version detection based on git tags
- **Improved**: Multiple output file generation support
- **Required**: Git repository for `gen` command (fails if not in git repo)

## License

MIT License
