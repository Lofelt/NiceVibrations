#!/bin/bash

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

STUDIO_LIBRARY_TAG=2.1.0

mkdir studio-content
cd studio-content
git init || exit_with_failure "git init failed"
git remote add -f origin git@github.com:Lofelt/studio-content.git || exit_with_failure "Fetching from remote failed"
git lfs install || exit_with_failure "Initializing git-lfs failed"
git config lfs.fetchinclude "Delivery/Studio-Library/"
git checkout tags/$STUDIO_LIBRARY_TAG -b version-$STUDIO_LIBRARY_TAG || exit_with_failure "Checking out tag branch failed"
git lfs pull || exit_with_failure "git lfs pull failed"
cd ..
