#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

# create folder where we place built archives
rm -r build
rm -r derived_data
mkdir build
mkdir build/XCArchives

echo "==========================================="
echo "Building archive for iOS Simulator"
echo "==========================================="

# build archive for simulators
xcodebuild clean archive \
  -project LofeltHaptics.xcodeproj \
  -scheme LofeltHaptics \
  -destination="iOS Simulator" \
  -archivePath build/XCArchives/ios_simulator \
  -configuration Release \
  -sdk iphonesimulator \
  -derivedDataPath derived_data \
  SKIP_INSTALL=NO \
  BUILD_LIBRARIES_FOR_DISTRIBUTION=YES \
  || exit_with_failure "Failed to build archive for iOS Simulator"

echo "==========================================="
echo "Building archive for iOS"
echo "==========================================="

#build archive for devices
xcodebuild clean archive \
  -project LofeltHaptics.xcodeproj \
  -scheme LofeltHaptics \
  -destination="iOS" \
  -archivePath build/XCArchives/ios \
  -configuration Release \
  -sdk iphoneos \
  -derivedDataPath derived_data \
  ENABLE_BITCODE=YES \
  BITCODE_GENERATION_MODE=bitcode \
  SKIP_INSTALL=NO \
  BUILD_LIBRARIES_FOR_DISTRIBUTION=YES \
  || exit_with_failure "Failed to build archive for iOS"

# create folder to store compiled framework
mkdir build/XCFramework

echo "==========================================="
echo "Creating XCFramework"
echo "==========================================="

#build the XCFramework from archives
xcodebuild -create-xcframework \
    -framework build/XCArchives/ios.xcarchive/Products/Library/Frameworks/LofeltHaptics.framework \
    -framework build/XCArchives/ios_simulator.xcarchive/Products/Library/Frameworks/LofeltHaptics.framework \
    -output build/XCFramework/LofeltHaptics.xcframework \
    || exit_with_failure "Failed to build XCFramework"
