#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

IPHONENAME=$1

# Validate arguments.
if [[ "$IPHONENAME" == "" ]]; then
    exit_with_failure "First argument should be the name of your attached iPhone. Run 'xcrun xctrace list devices' to get its name"
fi

xcodebuild test -project LofeltHaptics.xcodeproj -scheme LofeltHapticsOnDeviceTests -destination "platform=iOS,name=$IPHONENAME"
