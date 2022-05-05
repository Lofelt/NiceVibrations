- [Overview](#overview)
- [Copying of Plugins and Audio/Haptic Example Files](#copying-of-plugins-and-audiohaptic-example-files)
- [Releasing the Unity Plugin to the Unity Asset Store](#releasing-the-unity-plugin-to-the-unity-asset-store)
- [Developing the Unity plugin](#developing-the-unity-plugin)
- [Profiling the Unity plugin](#profiling-the-unity-plugin)
- [API Documentation](#api-documentation)
- [Unit Tests](#unit-tests)
  - [Running Tests](#running-tests)
    - [From Unity](#from-unity)
    - [From the Command Line](#from-the-command-line)
  - [Writing Tests](#writing-tests)

# Overview

This folder contains our Unity plugin. It consists of:
- The plugin script code (in `Assets/NiceVibrations/Scripts/`)
- The iOS and Android plugins from our Lofelt SDK, which are used by the script
  code (in `Assets/NiceVibrations/Plugins`)
- Audio and haptic example files (in `Assets/NiceVibrations/HapticSamples/`)
- Example scene (in `Assets/NiceVibrations/Demo/`)

# Copying of Plugins and Audio/Haptic Example Files

Neither the iOS and Android plugins nor the audio/haptic example files are part of this
repository, they need to be copied from other locations.

For the iOS and Android plugins, `build.sh android` and `build.sh ios` in the root of the
repository copy them into the right location.

For the audio/haptic example files, `copy-audio-haptic-examples.sh` copies them into
the right location. For this to work, the `studio-content` repository needs to exist as a
subfolder here. This can be achieved in two ways:
- If you have an existing checkout of that repository somewhere else on your machine, you
  can create a symlink
- You can run `fetch-audio-haptic-examples.sh`, which will clone the relevant parts of the
  repository into the `studio-content` subfolder

# Releasing the Unity Plugin to the Unity Asset Store

Before releasing the Unity plugin, some files need to be copied, as described in
the previous section. `create-release-zip.sh` copies all files and creates a zipped
version of our Unity plugin that is ready to be uploaded to the Asset Store. That
script is run by a [GitHub Actions workflow for releasing the SDK and Unity](../../.github/workflows/release-sdk-and-unity.yml),
which then attaches the Unity ZIP file to the GitHub release.

From that ZIP file, the Unity plugin can be uploaded to the Asset store as described
in [this Confluence page](https://lofelt.atlassian.net/wiki/spaces/PD/pages/1348108415/Release+Unity+Plugin+package).

Everything inside `interfaces/unity/NiceVibrations/Assets/NiceVibrations/` goes to the Asset Store. Everything outside doesn't go.

# Developing the Unity plugin

Under `interfaces/unity/NiceVibrations/Assets/Development` there is a scene that can be used to quickly implement and test new features. As the bigger `Demo` scene requires much more design work and doesn't cover all usecases.

Simply create new tab for any new features that don't fit the previous ones.

# Profiling the Unity plugin

Under `interfaces/unity/NiceVibrations/Assets/Profiling` there are scenes for profiling our Unity plugin. Inside it you will find both a baseline and main profiling scene. You can run the baseline scene on an iPhone (for example) and use the Debug Navigator in Xcode to measure CPU, memory and energy impact. Then you can do the same measurement with the main profiling scene having kicked off one of the tests from the phone. The difference between the measurements then, is the impact of our plugin on a game under certain conditions.

The baseline scene contains redundant game objects (dead buttons, game objects with no scripts applied) because the idea is to make it match the main profiling scene as closely as possible.

You can run the measurements for as long as your profiling use case requires. There are no measurement durations imposed by the code.

# API Documentation

We use Doxygen to generate HTML API documentation from the C# source code. This involves the
following files:
- `Doxygen.conf`: The configuration file that controls how Doxygen creates the API documentation
- `Mainpage.dox`: The content of the main page that is shown when first opening the API documentation
- `Lofelt_Logo.png`: The Lofelt logo that is included in the API documentation
- `generate-api-docs.sh`: A script that generates the API documentation by running Doxygen
- `doxygen/html/`: The output directory into which the generated API documentation will be put

The executables `doxygen` and `dot` (from the `graphviz` package) need to be installed on the system.
On macOS, they can be installed with `brew`:
```
brew install doxygen graphviz
```

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