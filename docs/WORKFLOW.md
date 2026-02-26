# Workflow Examples

## SemVer Release Pipeline

For traditional semantic versioning with candidates:

```bash
# Initialize SemVer project
bump init

# Develop with automatic git SHA suffixes
bump gen --lang c --output version.h  # Generates: 0.0.0+a1b2c3d (untagged)

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
bump gen --lang c --output version.h
bump gen --lang go --output version.go
# Build and deploy...
```

## CalVer Release Pipeline

For date-based versioning ideal for continuous deployment:

```bash
# Initialize CalVer project
bump init --calver

# Each day gets a new version automatically
bump --calendar               # Generates: 2026.02.25
git commit -am "Release $(bump -p)"
bump tag
bump gen --lang python --output version.py
# Build and deploy...

# Multiple releases same day? Automatic revision increment:
bump --calendar               # 2026.02.25-1
git commit -am "Hotfix $(bump -p)"
bump tag
bump gen --lang python --output version.py
# Build and deploy...
```

**PRO TIP**: Add generated version files to `.gitignore` to avoid "behind by one" issues
