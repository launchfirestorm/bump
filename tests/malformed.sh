#!/usr/bin/env bash

set -euo pipefail

# Integration tests for malformed and legacy bump.toml files.
# Covers v6 schema rejection, TOML syntax errors, and missing required fields.

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIXTURES="$ROOT/tests/fixtures/malformed"
cd "$ROOT"

if [[ -n "${BUMP_BIN:-}" ]]; then
    :
elif [[ -n "${CARGO_TARGET_DIR:-}" && -x "${CARGO_TARGET_DIR}/release/bump" ]]; then
    BUMP_BIN="${CARGO_TARGET_DIR}/release/bump"
else
    BUMP_BIN="$ROOT/target/release/bump"
fi

bump() {
    "$BUMP_BIN" "$@"
}

assert_fails() {
    local name="$1"
    local pattern="$2"
    local bumpfile="$3"
    local output
    local status=0

    echo "[$name]"
    set +e
    output="$(bump print "$bumpfile" 2>&1)"
    status=$?
    set -e

    if [[ "$status" -eq 0 ]]; then
        echo "expected failure, but command succeeded"
        echo "output: $output"
        exit 1
    fi

    if [[ "$output" != *"$pattern"* ]]; then
        echo "expected stderr/stdout to contain: $pattern"
        echo "got: $output"
        exit 1
    fi

    echo "ok"
    echo
}

assert_prints() {
    local name="$1"
    local expected="$2"
    local bumpfile="$3"
    local actual

    echo "[$name]"
    actual="$(bump print "$bumpfile")"
    echo "expected: $expected"
    echo "actual:   $actual"
    if [[ "$actual" != "$expected" ]]; then
        exit 1
    fi
    echo "ok"
    echo
}

assert_warns_and_prints() {
    local name="$1"
    local warn_pattern="$2"
    local expected="$3"
    local bumpfile="$4"
    local output
    local status=0

    echo "[$name]"
    set +e
    output="$(bump print "$bumpfile" 2>&1)"
    status=$?
    set -e

    if [[ "$status" -ne 0 ]]; then
        echo "expected success with warning, but command failed"
        echo "output: $output"
        exit 1
    fi

    if [[ "$output" != *"$warn_pattern"* ]]; then
        echo "expected warning containing: $warn_pattern"
        echo "got: $output"
        exit 1
    fi

    local actual="${output##*$'\n'}"
    echo "expected: $expected"
    echo "actual:   $actual"
    if [[ "$actual" != "$expected" ]]; then
        exit 1
    fi
    echo "ok"
    echo
}

# Legacy bump v6 schemas

assert_fails \
    "v6-semver" \
    "'base' table not found" \
    "$FIXTURES/v6-semver.toml"

assert_fails \
    "v6-calver" \
    "'base' table not found" \
    "$FIXTURES/v6-calver.toml"

# TOML syntax and structural errors

assert_fails \
    "invalid-toml" \
    "Failed to parse TOML document" \
    "$FIXTURES/invalid-toml.toml"

assert_fails \
    "missing-base" \
    "'base' table not found" \
    "$FIXTURES/missing-base.toml"

assert_fails \
    "base-not-table" \
    "'base' table not found" \
    "$FIXTURES/base-not-table.toml"

assert_fails \
    "missing-file" \
    "Configuration file not found" \
    "$FIXTURES/does-not-exist.toml"

# v7 schema validation (deserialization)

assert_fails \
    "missing-prefix" \
    "missing field \`prefix\`" \
    "$FIXTURES/missing-prefix.toml"

assert_fails \
    "missing-major" \
    "missing field \`major\`" \
    "$FIXTURES/missing-major.toml"

assert_fails \
    "missing-phase" \
    "missing field \`phase\`" \
    "$FIXTURES/missing-phase.toml"

assert_fails \
    "bad-label-position" \
    "unknown variant \`middle\`" \
    "$FIXTURES/bad-label-position.toml"

# Compatibility warnings and valid input

assert_warns_and_prints \
    "semver-with-calver-keys" \
    "found calver keys (year/month/day)" \
    "v2020.1.1" \
    "$FIXTURES/semver-with-calver-keys.toml"

assert_prints \
    "valid" \
    "v0.1.0" \
    "$FIXTURES/valid.toml"

echo "All malformed bumpfile tests passed."
