#!/bin/bash

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

if [[ "$OSTYPE" == "darwin"* ]]; then
    cargo build --package unity-editor-plugin --target aarch64-apple-darwin --release || exit_with_failure "Failed to build unity editor plugin for macOS ARM64"
    cargo build --package unity-editor-plugin --target x86_64-apple-darwin --release || exit_with_failure "Failed to build Unity editor plugin for macOS Intel"

    mkdir -p ../target/universal/release/
    lipo -output ../target/universal/release/libnice_vibrations_editor_plugin.dylib \
         -create ../target/aarch64-apple-darwin/release/libnice_vibrations_editor_plugin.dylib ../target/x86_64-apple-darwin/release/libnice_vibrations_editor_plugin.dylib
elif [[ "$OSTYPE" == "msys"* ]]; then
    cargo build --package unity-editor-plugin --release || exit_with_failure "Failed to build shared library for Windows"
else
    exit_with_failure "Unsupported host OS"
fi