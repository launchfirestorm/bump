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

## Usage

```bash
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
