#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}


echo "==========================================="
echo "Cleaning"
echo "==========================================="
./gradlew clean || exit_with_failure "Failed to clean the build"
rm -r javadoc/

echo "==========================================="
echo "Build Pass 1"
echo "==========================================="
./gradlew assembleRelease || exit_with_failure "Failed to build the library, first pass"

echo "==========================================="
echo "Build Pass 2"
echo "==========================================="
# The JNI libraries are missing from the AAR file in the first build, so build twice.
# See https://github.com/mozilla/rust-android-gradle/issues/43
./gradlew assembleRelease || exit_with_failure "Failed to build the library, second pass"

# Verify that the AAR file indeed contains the JNI libraries by checking its size.
# Without the JNI libraries, the AAR file is less than 10kb, with them it's more than
# 400kb.
AAR_FILE_SIZE=$(wc -c < "LofeltHaptics/build/outputs/aar/LofeltHaptics-release.aar")
if (( AAR_FILE_SIZE < 400000 )); then
    exit_with_failure "AAR file seems to be missing the JNI libraries"
fi

echo "==========================================="
echo "API Documentation"
echo "==========================================="
./gradlew generateJavadoc || exit_with_failure "Failed to build API documentation"

