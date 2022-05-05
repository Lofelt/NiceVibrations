#!/bin/sh

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

exit_with_usage_error()
{
    echo $1
    echo "Usage: bump-unity-version.sh <full-version> <short-version>"
    echo "Example: bump-unity-version.sh 4.0.0-alpha.2 4.0.0"
    exit 1;
}

FULL_VERSION=$1
SHORT_VERSION=$2

[[ -n "$FULL_VERSION" ]] || exit_with_usage_error "No full version given."
[[ -n "$SHORT_VERSION" ]] || exit_with_usage_error "No short version given."

[[ -z $(git status -s) ]] || exit_with_failure "git working directory not clean"

echo "Bumping version to $FULL_VERSION / $SHORT_VERSION..."

echo "➡️ Replacing versions in the files"

# The Unity project version needs to be the short version, as otherwise Unity's iOS build will fail:
#   UnityException: iOS Version has not been set up correctly, it must consist only of '.'s and
#   numbers, must begin and end with a number and be no longer than 18 characters the currently
#   set version string "4.0.0-alpha.0" contains invalid characters.
sed -i '' 's/  bundleVersion: .*/  bundleVersion: '$SHORT_VERSION'/' interfaces/unity/NiceVibrations/ProjectSettings/ProjectSettings.asset || exit_with_failure "Changing ProjectSettings.asset failed"

sed -i '' 's/^v\(.*\),\(.*\)v\(.*\)/v'$FULL_VERSION',\2v\3/' interfaces/unity/NiceVibrations/Assets/NiceVibrations/readme.txt || exit_with_failure "Changing readme.txt failed"
sed -i '' 's/  Version: v.*/  Version: v'$FULL_VERSION'/' interfaces/unity/NiceVibrations/Assets/NiceVibrations/Demo/NiceVibrationsDemo.unity || exit_with_failure "Changing NiceVibrationsDemo.unity failed"
sed -i '' 's/  m_Text: v[0-9]\.[0-9].*/  m_Text: v'$FULL_VERSION'/' interfaces/unity/NiceVibrations/Assets/NiceVibrations/Demo/NiceVibrationsDemo.unity || exit_with_failure "Changing NiceVibrationsDemo.unity failed"
sed -i '' 's/\(PROJECT_NUMBER.*=\).*/\1 v'$FULL_VERSION'/' interfaces/unity/Doxygen.conf || exit_with_failure "Changing Doxygen.conf failed"

echo "➡️ Committing version changes"
git add \
    interfaces/unity/NiceVibrations/Assets/NiceVibrations/readme.txt \
    interfaces/unity/NiceVibrations/Assets/NiceVibrations/Demo/NiceVibrationsDemo.unity \
    interfaces/unity/NiceVibrations/ProjectSettings/ProjectSettings.asset \
    interfaces/unity/Doxygen.conf
git commit -m "Bump Unity version number to $FULL_VERSION" ||  exit_with_failure "Failed to commit changes"


