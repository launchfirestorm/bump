# Configuration Reference

Bump uses a TOML configuration file (`bump.toml`) to manage versioning settings. You can choose between **SemVer** (Semantic Versioning) or **CalVer** (Calendar Versioning) when initializing.

**Note**: All comments in your `bump.toml` are preserved when the file is updated!

## SemVer Configuration

### Example Configuration

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
#  - "git_sha"  : append 7 char sha1 of the current commit (default)
#  - "branch"   : append the current git branch name
#  - "full"     : append <branch>_<sha1>
#  - "distance" : append commits since the latest reachable tag (pair with --label)
[semver.development]
promotion = "git_sha"
delimiter = "+"
```

### SemVer Options

#### `[semver.format]`
- **`prefix`**: Version tag prefix (e.g., "v", "release-", or empty string "")
- **`delimiter`**: Separator between version components (default: ".")
- **`timestamp`**: strftime format string for timestamps in generated files (optional)

#### `[semver.version]`
Current version numbers. This section is automatically updated by bump commands.

- **`major`**: Major version number
- **`minor`**: Minor version number
- **`patch`**: Patch version number
- **`candidate`**: Candidate version number (0 for non-candidate releases)

#### `[semver.candidate]`
- **`promotion`**: Which version component to bump when promoting to candidate
  - `"minor"`: Increment minor, zero patch (default)
  - `"major"`: Increment major, zero minor and patch
  - `"patch"`: Increment patch only
- **`delimiter`**: Separator for candidate versions (default: "-rc")

#### `[semver.development]`
- **`promotion`**: Strategy for development version suffixes
  - `"git_sha"`: Append 7-character commit SHA (default)
  - `"branch"`: Append current git branch name
  - `"full"`: Append `<branch>_<sha>`
  - `"distance"`: Append the number of commits since the latest reachable tag
    (`git describe`-style), falling back to the total commit count when no tag
    exists. Monotonic along a single line of history, computed at print time with
    no stored state — useful for telling dev builds apart at a glance.
- **`delimiter`**: Separator for development versions (default: "+"). Use `"-"`
  to place the suffix in the SemVer pre-release slot so it participates in
  version ordering.

##### Phase labels with `--label`
`--label <name>` prefixes the development suffix with a phase name, so a single
`bump.toml` can serve every branch in a gitflow-style workflow without edits:

```bash
bump --print-full --label dev    # develop  -> 1.7.0-dev.4
bump --print-full --label rc     # release  -> 1.7.0-rc.3
```

The label is only applied to development (untagged) builds; a tagged release
commit always prints the bare base version. Pair `promotion = "distance"` with
`delimiter = "-"` to get sortable, human-readable dev versions like `1.7.0-dev.4`.

### SemVer Commands

```bash
# Bump version numbers (updates bumpfile)
bump --major     # 1.0.0 -> 2.0.0
bump --minor     # 1.0.0 -> 1.1.0  
bump --patch     # 1.0.0 -> 1.0.1
bump --candidate # 1.0.0 -> 1.1.0-rc1 (or increment rc if already candidate)
bump --release   # 1.1.0-rc1 -> 1.1.0 (promote candidate to release)

bump --prefix "release-" --major [BUMPFILE]  # Uses "release-" instead of configured prefix
```

---

## CalVer Configuration

### Example Configuration

```toml
#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

# format will drive version section below
# - remove optional fields to change format
[calver.format]
prefix = ""         # defaults to empty string (no prefix)
delimiter = "."
year = "%Y"        # strftime 4 digit year
month = "%m"       # [optional] strftime zero padded month
day = "%d"         # [optional] strftime zero padded day

# NOTE: This section is modified by the bump command
[calver.version]
year = "2026"
month = "02"
day = "25"

# Same-day bumps increment revision number
# revision is modified by the bump command
[calver.conflict]
revision = 0
delimiter = "-"
```

### CalVer Options

#### `[calver.format]`
- **`prefix`**: Version tag prefix (e.g., "v", "release-", or empty string "")
- **`delimiter`**: Separator between date components (default: ".")
- **`year`**: strftime format for year (e.g., "%Y" for 4-digit year, "%y" for 2-digit)
- **`month`**: strftime format for month (optional, e.g., "%m" for zero-padded month)
- **`day`**: strftime format for day (optional, e.g., "%d" for zero-padded day)

#### `[calver.version]`
Current date components. This section is automatically updated by the `bump --calendar` command.

- **`year`**: Current year string (e.g., "2026")
- **`month`**: Current month string (optional, e.g., "02")
- **`day`**: Current day string (optional, e.g., "25")

#### `[calver.conflict]`
- **`revision`**: Revision number for same-day releases (automatically incremented)
  - Set to 0 for first release of the day
  - Increments to 1, 2, 3... for subsequent same-day bumps
  - Resets to 0 on new date
- **`delimiter`**: Separator for revision suffix (default: "-")

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

---

## Version Scheme Comparison

| Feature | SemVer | CalVer |
|---------|--------|--------|
| **Format** | major.minor.patch | Customizable date format |
| **Example** | v1.2.3, v1.2.0-rc1 | 2026.02.25, 2026.02.25-1 |
| **Best For** | Libraries, APIs, traditional releases | SaaS, continuous deployment |
| **Bumping** | --major, --minor, --patch | --calendar |
| **Candidates** | Yes (--candidate, --release) | No (use revision for same-day releases) |
| **Generated Code** | All version components | VERSION_STRING only |
| **Conflict Resolution** | N/A | Automatic revision increment |

---

## Global Options

All commands support the following global options:

- **BUMPFILE** (positional, default: `bump.toml`): Path to the configuration file to read version from (specified at the end)
  - Example: `bump --print custom.toml`
  - Example: `bump init config/version.toml`
- **--prefix PREFIX**: Override the prefix for version tags (e.g., 'v', 'release-', or empty string '')
- **Print flags** (all outputs are written **without a trailing newline**)
  - `--print`: Root version (prefix + candidate suffix if present)
  - `--print-base`: Base semver only: `MAJOR.MINOR.PATCH` (**SemVer only**)
  - `--print-full`: Full version with development suffix policy (`git_sha`, `branch`, or `full`)
  - `--print-with-timestamp`: Full version + configured build timestamp (**SemVer only**)
