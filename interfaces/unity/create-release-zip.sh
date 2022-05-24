#! /bin/sh

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

UNITY_VERSION=$1
[ "$2" != "--skip-unity-editor-plugin" ] ; SKIP_UNITY_EDITOR_PLUGIN=$?

[[ -n "$UNITY_VERSION" ]] || exit_with_failure "No version passed as first argument"
[ -d "NiceVibrations/Assets/NiceVibrations/Plugins/iOS/LofeltHaptics.framework" ] || exit_with_failure "iOS framework missing"
[ -f "NiceVibrations/Assets/NiceVibrations/Plugins/Android/libs/LofeltHaptics.aar" ] || exit_with_failure "Android library missing"

if [[ $SKIP_UNITY_EDITOR_PLUGIN -eq 0 ]] ; then
    [ -f "NiceVibrations/Assets/NiceVibrations/Plugins/Windows/x64/nice_vibrations_editor_plugin.dll" ] \
    || exit_with_failure "Windows .dll missing. If you want to ignore this step, run this script with '--skip-unity-editor-plugin' flag."
    [ -f "NiceVibrations/Assets/NiceVibrations/Plugins/macOS/libnice_vibrations_editor_plugin.dylib" ] \
    || exit_with_failure "macOS .dylib missing. If you want to ignore this step, run this script with '--skip-unity-editor-plugin' flag."
fi

if [[ -z "$CI" ]] ; then
    echo "⚠️ It appears you're running this script on your own machine."
    echo "⚠️ Be aware that the created ZIP file will include files ignored by git,"
    echo "⚠️ such as the asset cache."
    echo "⚠️ To avoid this, either run this script on a fresh git clone, or clean"
    echo "⚠️ this directory with 'git clean -dxf --dry-run .'."
fi

echo "➡️ Copying license file"
cp ../../licenses/3RD-PARTY-LICENSES.md NiceVibrations/Assets/NiceVibrations/ || exit_with_failure "Unable to copy the license file"

echo "➡️ Generating API documentation"
./generate-api-docs.sh || exit_with_failure "Generating API documentation failed"

echo "➡️ Creating ZIP file for main asset"
mkdir -p ../../release
zip --verbose --recurse-paths -X "../../release/unity-$UNITY_VERSION.zip" \
    NiceVibrations \
    --exclude "*.gitignore" \
    --exclude "*DS_Store" || exit_with_failure "Creating .zip for main asset failed"

echo "➡️ Creating ZIP file for API documentation"
cd doxygen/html || exit_with_failure "Failed to change to directory of generated API docs"
zip --verbose --recurse-paths -X "../../../../release/unity-api-docs-$UNITY_VERSION.zip" \
    * \
    --exclude "*.gitignore" \
    --exclude "*DS_Store" || exit_with_failure "Creating .zip for API documentation failed"
cd ../../


