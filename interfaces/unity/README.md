- [Overview](#overview)
- [Development](#development)
  - [Unity project](#unity-project)
    - [Development](#development-1)
    - [NiceVibrations](#nicevibrations)
    - [Profiling](#profiling)
    - [Tests](#tests)
  - [SDK changes](#sdk-changes)
  - [⚠️ Git considerations](#️-git-considerations)
- [API Documentation](#api-documentation)
- [Unit Tests](#unit-tests)
  - [Running Tests](#running-tests)
    - [From Unity](#from-unity)
    - [From the Command Line](#from-the-command-line)
  - [Writing Tests](#writing-tests)

# Overview

This folder contains the Nice Vibrations Unity asset. It consists of:
- The plugin script code (in `Assets/NiceVibrations/Scripts/`)
- The iOS and Android plugins from our Lofelt SDK, which are used by the script
  code (in `Assets/NiceVibrations/Plugins`)
- Audio and haptic example files (in `Assets/NiceVibrations/HapticSamples/`)
- Example scene (in `Assets/NiceVibrations/Demo/`)

# Development

## Unity project

A unity project is located at `interfaces/unity/NiceVibrations/`. Inside the project `Assets` there are folders used for different goals.

### Development

This folder contains a scene that can be used to quickly implement and test new features. The "bigger" `NiceVibrations/Demo` scene requires much more design work and doesn't cover all use cases for now.

Simply create new tab for any new features that don't fit in the existing ones.

### NiceVibrations

This folder contains the NiceVibrations asset that is shipped to users. Updates, fixes and new features to be released should be implemented here.

### Profiling

This folder contain scenes for profiling the Nice Vibrations asset. Inside you will find both a baseline and main profiling scene. You can run the baseline scene on an iPhone (for example) and use the Debug Navigator in Xcode to measure CPU, memory and energy impact. Then you can do the same measurement with the main profiling scene having kicked off one of the tests from the phone. The difference between the measurements then, is the impact of our plugin on a game under certain conditions.

The baseline scene contains redundant game objects (dead buttons, game objects with no scripts applied) because the idea is to make it match the main profiling scene as closely as possible.

You can run the measurements for as long as your profiling use case requires. There are no measurement durations imposed by the code.

### Tests

Contains scripts for unit testing. Check the [Test section](#unit-tests) to know more about Unity unit testing.

## SDK changes

When changes are done for the iOS, Android or the Unity Editor plugin host platforms, you will need to run `build-platform.sh` to make sure that the latest changes from those libraries are then copied into the appropriate Unity asset folder.

## ⚠️ Git considerations

If you're using Git, some .meta files associated with the shared libraries inside `interfaces/unity/NiceVibrations/Assets/NiceVibrations/Plugins` have important information regarding which architectures to build for, etc. 

If you plan to make commits, please check that you are intentionally making changes in .meta files for Android library, iOS framework, .so and .dll before making commits to those files.

A common example of this happens when using macOS, where running `build.sh` doesn't cross-compile for Windows. And so, a .dll file won't be in the Plugins folder and Unity removes the .meta file associated with the .dll.

# API Documentation

We use Doxygen to generate HTML API documentation from the C# source code. This involves the
following files:
- `Doxygen.conf`: The configuration file that controls how Doxygen creates the API documentation
- `Mainpage.dox`: The content of the main page that is shown when first opening the API documentation
- `Lofelt_Logo.png`: The Lofelt logo that is included in the API documentation
- `generate-api-docs.sh`: A script that generates the API documentation by running Doxygen
- `doxygen/html/`: The output directory into which the generated API documentation will be put in.

`create-release-zip.sh` creates two ZIP files, one for the Unity package and one for the API
documentation.

# Unit Tests

For more information about unit tests in Unity, see the
[Unity documentation](https://docs.unity3d.com/Packages/com.unity.test-framework@1.1/manual/index.html).

## Running Tests

The tests can be run from within Unity or from the command line.

### From Unity
In Unity, select "Windows->General->Test Runner" from the menu, which opens a new test runner
window. In the test runner, you can select and run individual tests, or run the whole test suite with
the "Run All" button.

### From the Command Line
You can run `ci-unity.sh` in the root folder of this repository to run the unit tests from the
command line.
Currently the script has limited support for detecting the location of the Unity executable. You can
manually specify it with `UNITY_EXECUTABLE=/path/to/my/Unity ci-unity.sh`.

## Writing Tests

The source code of the tests is in `NiceVibrations/Assets/Tests/Tests.cs`. To access non-public
classes and members in `NiceVibrations/Assets/NiceVibrations/Scripts/Components`, mark the class
or member with `internal`. Internal entities are available in the tests due to a setting in
`NiceVibrations/Assets/NiceVibrations/Scripts/Components/AssemblyInfo.cs`.
