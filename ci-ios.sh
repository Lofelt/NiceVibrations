#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

echo "➡️ Building"
./build-platform.sh ios || exit_with_failure "Building failed"

cd examples/ios
echo "➡️ Archiving example"
./archive-example.sh || exit_with_failure "Archiving example failed"
cd ../..
