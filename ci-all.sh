#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

./ci-rust-core.sh || exit_with_failure "CI script for Rust core failed"
./ci-unity.sh || exit_with_failure "CI script for Unity failed"
./ci-ios.sh || exit_with_failure "CI script for iOS failed"
./ci-android.sh || exit_with_failure "CI script for Android failed"
