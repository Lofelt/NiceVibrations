#!/bin/sh
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

exit_with_usage_error()
{
    echo $1
    echo "Usage: bump-sdk-version.sh <full-version> <short-version>"
    echo "Example: bump-sdk-version.sh 1.3.0-alpha.0 1.3.0"
    exit 1;
}

FULL_VERSION=$1
SHORT_VERSION=$2

[[ -n "$FULL_VERSION" ]] || exit_with_usage_error "No full version given."
[[ -n "$SHORT_VERSION" ]] || exit_with_usage_error "No short version given."

[[ -z $(git status -s) ]] || exit_with_failure "git working directory not clean"

echo "Bumping version to $FULL_VERSION / $SHORT_VERSION..."

echo "➡️ Replacing versions in the files"

# Cargo.toml can contain multiple lines with 'version = "xyz"'. Match only those where 'xyz'
# is '[1-9].[0-9].[0-9].*', which at the time of writing is only our version number.
# A cleaner solution would be to only replace the first match, but on macOS, which doesn't
# have GNU sed, that's complicated, see https://stackoverflow.com/a/11458836/1005419.
sed -i '' 's/version = "[1-9].[0-9].[0-9].*"/version = "'$FULL_VERSION'"/' core/lib/Cargo.toml || exit_with_failure "Changing lib/Cargo.toml failed"
sed -i '' 's/version = "[1-9].[0-9].[0-9].*"/version = "'$FULL_VERSION'"/' core/api/Cargo.toml || exit_with_failure "Changing api/Cargo.toml failed"

sed -i '' 's/versionName ".*"/versionName "'$FULL_VERSION'"/' interfaces/android/LofeltHaptics/LofeltHaptics/build.gradle  || exit_with_failure "Changing build.gradle failed"

sed -i '' 's/^v\(.*\),\(.*\)v\(.*\)/v\1,\2v'$FULL_VERSION'/' interfaces/unity/NiceVibrations/Assets/NiceVibrations/readme.txt || exit_with_failure "Changing readme.txt failed"

# We cannot indicate beta or anything other than major.minor.patch as the App Store will reject that
# deviates from it so just enter the version that the beta targets.
sed -i '' 's/DYLIB_COMPATIBILITY_VERSION = .*;/DYLIB_COMPATIBILITY_VERSION = '$SHORT_VERSION';/' interfaces/ios/LofeltHaptics/LofeltHaptics.xcodeproj/project.pbxproj || exit_with_failure "Changing project.pbxproj failed"
sed -i '' 's/DYLIB_CURRENT_VERSION = .*;/DYLIB_CURRENT_VERSION = '$SHORT_VERSION';/' interfaces/ios/LofeltHaptics/LofeltHaptics.xcodeproj/project.pbxproj || exit_with_failure "Changing project.pbxproj failed"
sed -i '' 's/MARKETING_VERSION = .*;/MARKETING_VERSION = '$SHORT_VERSION';/' interfaces/ios/LofeltHaptics/LofeltHaptics.xcodeproj/project.pbxproj || exit_with_failure "Changing project.pbxproj failed"

echo "➡️ Rebuilding to update Cargo.lock"
./ci-rust-core.sh || exit_with_failure "Updating Cargo.lock failed"

echo "➡️ Committing version changes"
git add \
    Cargo.lock \
    core/lib/Cargo.toml \
    core/api/Cargo.toml \
    interfaces/android/LofeltHaptics/LofeltHaptics/build.gradle \
    interfaces/unity/NiceVibrations/Assets/NiceVibrations/readme.txt \
    interfaces/ios/LofeltHaptics/LofeltHaptics.xcodeproj/project.pbxproj
git commit -m "Bump SDK version number to $FULL_VERSION" ||  exit_with_failure "Failed to commit changes"


