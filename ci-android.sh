#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

[ "$1" != "--skip-connected-check" ] ; SKIP_CONNECTED_CHECK=$?

echo "➡️ Building"
./build-platform.sh android || exit_with_failure "Building failed"

cd interfaces/android/LofeltHaptics

echo "➡️ Linting"
./gradlew lint || exit_with_failure "Linting failed"

if [[ $SKIP_CONNECTED_CHECK -eq 0 ]] ; then
    echo "➡️ Running tests"
    ./gradlew connectedCheck || exit_with_failure "Running tests failed"
fi

cd ../../..

cd examples/android/LofeltHapticsExamplePreAuthored

echo "➡️ Building example"
./gradlew build || exit_with_failure "Building example failed"

echo "➡️ Linting example"
./gradlew lint || exit_with_failure "Linting example failed"
cd ../../..
