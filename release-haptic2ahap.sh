# This script releases the haptic2ahap app.
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

package_name=haptic2ahap
binary_name=haptic2ahap
version=$1

[[ -n "$version" ]] || exit_with_failure "No version passed as first argument"

echo "Releasing package $package_name version $version as binary $binary_name"

echo "➡️ Cleaning release directory"
rm -rf release/

echo "➡️ Building $package_name"
cargo build --package $package_name --release || exit_with_failure "Building $package_name failed"

# Zip everything up and put the result in a new 'release/' directory
echo "➡️ Creating .zip"
mkdir -p release
zip --verbose --recurse-paths --junk-paths -X "release/$package_name-$version.zip" \
    target/release/$binary_name \
    --exclude "*.gitignore" --exclude "*DS_Store" || exit_with_failure "Creating .zip failed"
