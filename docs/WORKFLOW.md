# Workflow Examples

## Single BUMPFILE Pipeline

For traditional semantic versioning with candidates:

### 1. Bump, tag, and push
```bash
bump --minor                    # using logic you can change this behavior 
bump update Cargo.toml          # if you need to
git add bump.toml               # add the changes to the index
git commit                      # capture the changes 
bump tag                        # name this commit to the version we just bumped
git push origin HEAD --tags     # push edits to remote
```

### 2. Now with tagged build (tag pipeline)
```bash
# Generate version file to inject into the build system (bake into lib/app)
bump gen --lang c --output version.h  # Generates: 0.1.0+a1b2c3d (untagged)

# NOTE: version.h should be git ignored

# Build your code 
<build tool> ...
```

### 3. Deploy / Release

## Multiple BUMPFILE Pipeline

The glory of `bump` is that you can version MUTLIPLE things at once! 

### 1. Bump, tag, and push
```bash
bump --minor lib1/bump.toml             # again logic can conditionally set this
bump --major app/component/bump.toml    # different parts of the repo can have different cadences
git add -u
git commit
bump tag
git push origin HEAD --tags
```