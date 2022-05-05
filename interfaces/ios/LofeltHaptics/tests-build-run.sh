#!/bin/bash

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

xcodebuild test \
-project LofeltHaptics.xcodeproj/ \
-scheme LofeltHapticsTestsMacOS \
-destination 'platform=OS X,arch=x86_64' \
-configuration Release \
  CODE_SIGN_IDENTITY="" \
  CODE_SIGNING_REQUIRED=NO \
  ENABLE_BITCODE=YES \
  BITCODE_GENERATION_MODE=bitcode || exit_with_failure "Running tests failed"
