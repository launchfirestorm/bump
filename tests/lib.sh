#!/usr/bin/env bash

# Shared helpers for bump integration tests.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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
