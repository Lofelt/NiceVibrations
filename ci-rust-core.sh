#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

echo "➡️ Checking code formatting"
cargo fmt -- --check || exit_with_failure "Checking code formatting failed"

echo "➡️ Running clippy linter"
cargo clippy --all-targets -- -D warnings || exit_with_failure "Running clippy linter failed"

echo "➡️ Building"
cargo build || exit_with_failure "Building failed"

echo "➡️ Building benchmarks"
cargo build --benches || exit_with_failure "Building benchmarks failed"

echo "➡️ Running tests"
cargo test || exit_with_failure "Running tests failed"

