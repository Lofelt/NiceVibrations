#!/bin/bash

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

echo "➡️ Building and copying artifacts"
./build-platform.sh unity-editor-plugin-host || exit_with_failure "Building failed"

echo "➡️ Running tests"
cd unity-editor-plugin
cargo test || exit_with_failure "Running tests failed"
cd ../
