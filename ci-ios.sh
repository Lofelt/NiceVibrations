#!/bin/bash

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

echo "➡️ Building"
./build.sh ios || exit_with_failure "Building failed"

cd interfaces/ios/LofeltHaptics
echo "➡️ Running tests"
./tests-build-run.sh || exit_with_failure "Running tests failed"
cd ../../..

cd examples/ios
echo "➡️ Archiving examples"
./archive-examples.sh || exit_with_failure "Archiving examples failed"
cd ../..
