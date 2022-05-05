#!/bin/bash

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

PLATFORM=$1
DEVICENAME=$2

# Validate arguments.
if [[ "$PLATFORM" != "ios" && "$PLATFORM" != "android" ]]; then
    exit_with_failure "First argument should be 'ios' or 'android'."
fi

if [[ "$PLATFORM" == "ios" && "$DEVICENAME" == "" ]]; then
    exit_with_failure "Second argument should be the name of your attached iPhone. Run 'xcrun xctrace list devices' to get its name"
fi

if [ "$PLATFORM" = "ios" ]; then
    echo "Running on-device tests for iOS"
    cd "interfaces/ios/LofeltHaptics"
    sh tests-run-on-device.sh "$DEVICENAME" || exit_with_failure "Running iOS tests failed"
    cd ../../..
elif [ "$PLATFORM" = "android" ]; then
    echo "Running on-device tests for Android"
    cd "interfaces/android/LofeltHaptics"
    ./gradlew connectedAndroidTest
    cd ../../..
fi
