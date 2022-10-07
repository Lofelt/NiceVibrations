#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

DEST_NICE_VIBRATIONS_ASSET_DIR=nice-vibrations-asset
SOURCE_NICE_VIBRATIONS_FILES_DIR=interfaces/unity/NiceVibrations/Assets/NiceVibrations
SOURCE_UNITY_DIR=interfaces/unity

echo "➡️ Cleaning Nice Vibrations asset artifacts"
rm -rf $DEST_NICE_VIBRATIONS_ASSET_DIR

sh build-platform.sh ios || exit 1
sh build-platform.sh android || exit 1
sh build-platform.sh unity-editor-plugin-host || exit 1

echo "➡️ Copying license file"
cp licenses/3RD-PARTY-LICENSES.md $SOURCE_NICE_VIBRATIONS_FILES_DIR/ || exit_with_failure "Unable to copy the license file"

echo "➡️ Generating API documentation"
cd $SOURCE_UNITY_DIR/
./generate-api-docs.sh || exit_with_failure "Generating API documentation failed"
cd -

echo "➡️ Creating folder for Nice Vibrations asset"
mkdir -p $DEST_NICE_VIBRATIONS_ASSET_DIR/ || exit_with_failure "Creating Nice Vibrations asset folder failed"
cp -rf $SOURCE_NICE_VIBRATIONS_FILES_DIR $DEST_NICE_VIBRATIONS_ASSET_DIR/ || exit_with_failure "Copying artifacts to Nice Vibrations asset folder failed"

echo "✅ Creating Nice Vibrations asset folder done!🎉"
