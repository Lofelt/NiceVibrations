#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

cd interfaces/unity

echo "➡️ Running unit tests"

if [[ -z "$UNITY_EXECUTABLE" ]]; then
    echo "Attempting to auto-detect Unity executable..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # TODO: Support different versions. Hardcode 2019.4.16f1 for now, as that's the minimum version
        #       we support in Nice Vibrations, so every developer should have it installed.
        UNITY_EXECUTABLE=~/Documents/Unity/2019.4.16f1/Unity.app/Contents/MacOS/Unity
        if [[ ! -r "$UNITY_EXECUTABLE" ]]; then
            UNITY_EXECUTABLE=/Applications/Unity/Hub/Editor/2019.4.16f1/Unity.app/Contents/MacOS/Unity
        fi
    else
        echo "⚠ Skipping unit tests, detecting the Unity executable on this OS not yet supported by this script."
        echo "⚠ You can manually set the UNITY_EXECUTABLE environment variable to the location of the Unity executable an re-run this script."
    fi
else
    echo "UNITY_EXECUTABLE set from outside to $UNITY_EXECUTABLE"
fi

if [[ -n "$UNITY_EXECUTABLE" ]]; then
    echo "Using Unity executable at $UNITY_EXECUTABLE"

    TEST_RESULT_FILENAME=test-results.xml
    rm -f NiceVibrations/$TEST_RESULT_FILENAME

    $UNITY_EXECUTABLE \
        -buildOSXUniversalPlayer \
        -projectPath NiceVibrations \
        -nographics \
        -runTests \
        -testPlatform PlayMode \
        -batchmode \
        -logFile - \
        -testResults $TEST_RESULT_FILENAME

    if [ $? -ne 0 ]; then
        if [[ -f NiceVibrations/$TEST_RESULT_FILENAME ]]; then
            echo
            echo
            echo "===================================================================="
            echo "UNITY TESTS FAILED, CONTENT OF $TEST_RESULT_FILENAME:"
            echo "===================================================================="
            cat NiceVibrations/$TEST_RESULT_FILENAME
            echo
        fi
        exit_with_failure "Unity tests failed!"
    else
        rm NiceVibrations/$TEST_RESULT_FILENAME
    fi
fi

echo "➡️ Generating API docs"
./generate-api-docs.sh || exit_with_failure "Generating API docs failed"

if [[ -z "$UNITY_EXECUTABLE" ]]; then
    echo "⚠ Unity unit tests were skipped, see log further above for details."
fi

cd ../..