# Workflow Examples

## Single BUMPFILE Pipeline

Use this for a single `bump.toml` at repository root.

### 1. Bump, update metadata, tag, and push

```bash
bump --minor
bump update Cargo.toml

git add bump.toml Cargo.toml
git commit -m "chore(release): bump version"

bump tag
git push origin HEAD --tags
```

### 2. Generate version files during builds

```bash
# Example: generate C header from current bumpfile state
bump gen --lang c --output version.h

# Then run your normal build
<build tool> ...
```

## SemVer Phase Workflow

```bash
# Start a release candidate phase
bump --phase rc         # e.g., 1.4.0 -> 1.4.0-rc.1

# Continue same phase
bump --phase            # e.g., 1.4.0-rc.2
bump --phase rc         # also increments when phase matches current

# Switch phase name
bump --phase beta       # e.g., 1.4.0-beta.1
```

## CalVer Workflow

Set `mode = "calver"` in your bumpfile first, then:

```bash
bump --calendar
```

If the date is unchanged, calendar bump increments `phase.distance`.

## Multiple BUMPFILE Pipeline

`bump` supports multiple version streams in one repository by passing the file path as the positional `BUMPFILE` argument.

```bash
bump --minor lib1/bump.toml
bump --major app/component/bump.toml

git add -u
git commit -m "chore(release): bump component versions"

# tag uses the default bumpfile unless an explicit BUMPFILE is provided
bump tag lib1/bump.toml
bump tag app/component/bump.toml

git push origin HEAD --tags
```

## CI-Friendly Version Output

```bash
bump print --only-base
bump print --full
bump print --with-suffix
```

All print commands emit without a trailing newline, so they are safe for shell substitution.