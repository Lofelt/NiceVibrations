#!/bin/bash

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

UNITY_HAPTIC_SAMPLES_DIRECTORY=NiceVibrations/Assets/NiceVibrations/HapticSamples
STUDIO_CONTENT_DIRECTORY=studio-content/Delivery/Studio-Library
STUDIO_TUTORIAL_DIRECTORY_NAME=/\[doc\]\Tutorial/

mkdir -p $UNITY_HAPTIC_SAMPLES_DIRECTORY || exit_with_failure "Unable to create HapticSamples folder"

cp -R "$STUDIO_CONTENT_DIRECTORY"/ $UNITY_HAPTIC_SAMPLES_DIRECTORY/ || exit_with_failure "Unable to copy to HapticSamples folder"

# delete all .lofelt project files
find $UNITY_HAPTIC_SAMPLES_DIRECTORY -type f -name '*.lofelt' -delete
# delete all .svg files
find $UNITY_HAPTIC_SAMPLES_DIRECTORY -type f -name '*.svg' -delete

# remove white spaces from sub-directories and files
# NOTE: requires `rename`, to install run `brew install rename`
set -o pipefail
find $UNITY_HAPTIC_SAMPLES_DIRECTORY/ -depth 1 -name "* *" -type d | rename 's/ //g' || exit_with_failure "Removing spaces from directories failed (1/2)"
find $UNITY_HAPTIC_SAMPLES_DIRECTORY/ -depth 2 -name "* *" -type d | rename 's/ //g' || exit_with_failure "Removing spaces from directories failed (2/2)"
find $UNITY_HAPTIC_SAMPLES_DIRECTORY -name "* *" -type f | rename 's/ //g' || exit_with_failure "Removing spaces from files failed"

# delete Studio tutorials folder
rm -rf "$UNITY_HAPTIC_SAMPLES_DIRECTORY$STUDIO_TUTORIAL_DIRECTORY_NAME"