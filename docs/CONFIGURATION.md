# Configuration Reference

`bump` uses one unified TOML schema in `bump.toml` for both SemVer and CalVer.

## BUMPFILE, can named anything default is `bump.toml`
```toml
#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)
#
# https://github.com/launchfirestorm/bump

[timestamp]
format = "%Y-%m-%d %H:%M:%S %Z"   # strftime syntax, used in file generation
last = "2026-06-05 19:06:16 UTC"

# NOTE: some fields are modified by bump
#   - mode: "semver" | "calver"
#   - minor is optional and can be removed if not needed
#   - patch is optional and can be removed if not needed
[base]
mode = "semver"
prefix = "v"
delimiter = "."
major = 0  
minor = 1
patch = 0

[phase]  
prefix = "-"
name = ""
delimiter = "-"
distance = 0

# suffix type:
#  - "git_sha"  : append 7 char sha1 of the current commit (default)
#  - "branch"   : append the current git branch name
[suffix]
mode = "git_sha"
delimiter = "+"

# printed label: shown but never tracked, useful for injecting dynamic values
#  - position: "before-base", "after-base", "before-phase", "after-phase"
[label]
position = "before-phase"
```

## Key Sections

### `[timestamp]`

- `format`: `strftime` format used when writing `timestamp.last`.
- `last`: updated on every bump operation.

### `[base]`

- `mode`: `semver` or `calver`.
- `prefix`: optional leading text (for example `v`).
- `delimiter`: separator for base components.
- `major`, `minor`, `patch`: numeric components.
- `minor` and `patch` are optional.

For compatibility, `year`, `month`, and `day` are accepted as aliases for
`major`, `minor`, and `patch` when loading.

### `[phase]`

- `prefix`: inserted before phase data (commonly `-`).
- `name`: phase label (for example `rc`, `beta`, or empty).
- `delimiter`: separator between `name` and `distance`.
- `distance`: phase counter.

### `[suffix]`

- `mode`: `git_sha` or `branch`.
- `delimiter`: separator before the suffix payload.

### `[label]`

- `position`: where `bump print --with-label <LABEL>` injects runtime label text.
- Label value is never written to the bumpfile.

## Mode-Specific Behavior

### SemVer mode

- Supported bump ops: `--major`, `--minor`, `--patch`, `--phase`.
- `--calendar` is rejected.
- Base format is `<major><delimiter><minor><delimiter><patch>`.

### CalVer mode

- Supported bump ops: `--calendar`, `--phase`.
- `--major`, `--minor`, and `--patch` are rejected.
- Month/day values are printed with zero padding in base output.

## Key Remapping Rules

When writing back to disk, keys are normalized to match `base.mode`.

- If `mode = "semver"`, stored keys become `major/minor/patch`.
- If `mode = "calver"`, stored keys become `year/month/day`.

Additional safety behavior:

- If `mode = "semver"` but the file contains `year/month/day`, a warning is
  emitted and keys are rewritten on save.

## Print Output Modes

Use the `print` subcommand. Flags are stackable except `--only-*` and `--full`:

```bash
Print [prefix][base][phase] from BUMPFILE without newline

Usage: bump print [OPTIONS] [BUMPFILE]

Arguments:
  [BUMPFILE]  Path to the configuration file [default: bump.toml]

Options:
      --only-prefix     Print [prefix] only
      --only-phase      Print [phase] only
      --only-base       Print [base] only
      --no-prefix       Omit [prefix]
      --no-phase        Omit [phase]
      --with-suffix     Append [suffix]
      --with-timestamp  Append [timestamp]
      --with-label      Inject LABEL at [label].position (not persisted)
      --full            Print full output; overrides all flags except --with-label
  -h, --help            Print help
```

All print variants emit output without a trailing newline.
