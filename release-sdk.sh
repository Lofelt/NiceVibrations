exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

SDK_VERSION=$1
[[ -n "$SDK_VERSION" ]] || exit_with_failure "No version passed as first argument"

if [[ -z "$CI" ]] ; then
    echo "⚠️ It appears you're running this script on your own machine."
    echo "⚠️ Be aware that the created ZIP file will include files ignored by git,"
    echo "⚠️ such as build artifacts, in the examples."
    echo "⚠️ To avoid this, either run this script on a fresh git clone, or clean"
    echo "⚠️ the examples directory with 'git clean -dxf --dry-run examples/'."
fi

echo "Cleaning release directory"
rm -rf release/

sh build.sh ios || exit_with_failure "Building for iOS failed"
sh build.sh android || exit_with_failure "Building for Android failed"

# Zip everything up and put the result in a new 'release/' directory
mkdir -p release
zip --verbose --recurse-paths -X "release/sdk-$SDK_VERSION.zip" \
    examples/android/LofeltHapticsExamplePreAuthored/ \
    examples/ios/LofeltHapticsExamplePreAuthored/ \
    examples/ios/LofeltHapticsExampleRealtime/ \
    ios-framework/ \
    android-library/ \
    licenses/ \
    --exclude "*.gitignore" --exclude "*DS_Store" --exclude "*.template" \
        || exit_with_failure "Creating .zip failed"
