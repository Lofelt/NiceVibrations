# This script releases the haptic2ahap app.

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

package_name=$1
binary_name=$2
version=$3

[[ -n "$package_name" ]] || exit_with_failure "No package name passed as first argument"
[[ -n "$binary_name" ]] || exit_with_failure "No binary name passed as second argument"
[[ -n "$version" ]] || exit_with_failure "No version passed as third argument"

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
