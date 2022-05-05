#!/bin/bash

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

[[ -d "../../ios-framework/Xcode12AndHigher/LofeltHaptics.xcframework" ]] || exit_with_failure "iOS framework has not been built"

echo "==========================================="
echo "Archiving pre-authored example"
echo "==========================================="
cd LofeltHapticsExamplePreAuthored/
xcodebuild clean archive \
    CODE_SIGN_IDENTITY="" \
    CODE_SIGNING_REQUIRED="NO" \
    CODE_SIGN_ENTITLEMENTS="" \
    CODE_SIGNING_ALLOWED="NO" \
    -configuration Release \
    -project LofeltHapticsExamplePreAuthored.xcodeproj || exit_with_failure "Failed to archive pre-authored example"
cd ..
