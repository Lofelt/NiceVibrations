#!/bin/bash

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

[ "$1" != "--skip-wasm" ] ; SKIP_WASM=$?

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

if [[ $SKIP_WASM -eq 0 ]] ; then
    cd interfaces/wasm

    echo "➡️ Running WASM tests"
    wasm-pack test --node || exit_with_failure "Running WASM tests failed"

    # This needs to be done after wasm-pack test, as on the CI, wasm-pack test will
    # automagically install the wasm32-unknown-unknown target required in this step.
    #
    # Having to run clippy for the WASM target is needed to also check files that are
    # conditionally compiled only for WASM.
    echo "➡️ Running clippy linter for WASM"
    cargo clippy --target wasm32-unknown-unknown -- -D warnings || exit_with_failure "Running clippy linter for WASM failed"

    cd ../..
fi
