![](https://github.com/Lofelt/NiceVibrations/workflows/ios/badge.svg)
![](https://github.com/Lofelt/NiceVibrations/workflows/android/badge.svg)
![](https://github.com/Lofelt/NiceVibrations/workflows/rust-core/badge.svg)

![Nice Vibrations 4 Logo](media/nice-vibrations-lofelt-logo-yellow.png)

# ℹ️ Contents

- [ℹ️ Contents](#ℹ️-contents)
- [🎮 What is Nice Vibrations?](#-what-is-nice-vibrations)
  - [Main features](#main-features)
- [📖 Documentation](#-documentation)
- [🆕 Getting Started](#-getting-started)
  - [Setting up the Development Environment on macOS](#setting-up-the-development-environment-on-macos)
    - [Lofelt Studio SDK](#lofelt-studio-sdk)
    - [Nice Vibrations](#nice-vibrations)
  - [Build and use Nice Vibrations Asset](#build-and-use-nice-vibrations-asset)
  - [Build Lofelt Studio SDK individually and run tests](#build-lofelt-studio-sdk-individually-and-run-tests)
  - [Build and Test Lofelt SDK for iOS](#build-and-test-lofelt-sdk-for-ios)
  - [Build and Test Lofelt SDK for Android](#build-and-test-lofelt-sdk-for-android)
  - [Build and Test Lofelt Unity Editor plugin host](#build-and-test-lofelt-unity-editor-plugin-host)
  - [Development](#development)
  - [Git rules](#git-rules)
- [🏗️ Continuous Integration](#️-continuous-integration)
  - [GitHub actions](#github-actions)
  - [Locally](#locally)
- [©️ License](#️-license)
- [✍️ Authors](#️-authors)

# 🎮 What is Nice Vibrations?

Built on top of Lofelt Studio SDK, Nice Vibrations is a Unity Asset which offers
a universal interface to trigger HD haptic feedback on supported Unity platforms
platforms at once, as well as ways to get exactly the right vibration on each
platform.

Nice Vibrations provides full support for:

🤖 Android

🍏 iOS haptic APIs (including iOS Core Haptics)

🎮 Gamepad rumble.

Nice Vibrations comes with a growing collection of royalty-free HD haptic (and
audio) assets ready for quick prototyping and testing. No design skills or tasks
required. The assets cover various use cases like Application UX, Game FX, Game
Objects, and more.

This repository contains both the open-source code of Lofelt Studio SDK and Nice
Vibrations.

## Main features

🕹 **Play and Stop**: play and stop complex HD haptic clips (.haptic files).

📂 **iOS improved performance**: load and play haptic clips 2x faster than on
Nice Vibrations 3.9

➰ **Loop**: loop a playing haptic clip.

👇 **Seek**: jump into any point of your haptic clip before or during playback
depending on what is happening in your game.

🔣 **Modulate Frequency or Amplitude**: adjust the strength or frequency of your
playing haptic clip in  reaction to other on screen events (distance between
characters, size of a collision), or just to create variation in the pattern
playback (repetitive actions like weapon firing or footsteps and jumps).

⏱ **Haptic Priority**: set the priority of your haptic clip. Lower priority will
be silenced if higher priority haptic clips are triggered at the same time.

💼 **Code Only API**: no need to be stuck in Unity and tied to MonoBehaviours.
With the code-only API, you can work in any development environment.

🖋 **MonoBehaviour API**: play haptics by just using the Unity Editor, without
writing a single line of C# code.

✒ **Platform Independence**: use the same C# and the same .haptic file on all
platforms, no need to write platform-specific code for Android or iOS.

📕 **Haptic Library**: no need to design your own haptics. A growing library of
haptic clips is included to get you started quickly and with limited experience.

🎚 **Global Haptic Level and Mute**: disable haptics entirely with one switch or
reduce the intensity of haptics across the whole application.

💁 **Fully Featured Demos**: demos included to help you learn how to apply all
of the features for your specific use case. These demos will also help you see
how to migrate your existing Nice Vibrations code to the new API’s.

🎗 **Haptic Presets Support**: trigger 9 predefined haptic patterns with a
single line of code.

🎳 **Gamepad Rumble**: play .haptic files on gamepads supported by Unity.

☎ **Fallback Support**: play basic haptics on older phones and operating
systems.

And more..

# 📖 Documentation

The [Nice Vibrations
documentation](https://github.com/Lofelt/NiceVibrations/wiki/1.-Nice-Vibrations-developer-documentation)
is available in this repository's Wiki. It explains how to integrate and use
Nice Vibrations with new or existing Unity projects. It also contains
documentation for the Lofelt SDK for
[iOS](https://github.com/Lofelt/NiceVibrations/wiki/2.-Lofelt-SDK-for-iOS) and
[Android](https://github.com/Lofelt/NiceVibrations/wiki/3.-Lofelt-SDK-for-Android).

# 🆕 Getting Started

## Setting up the Development Environment on macOS

Nice Vibrations depends on the cross-platform Lofelt Studio SDK. This means that
the Lofelt Studio SDK needs to be build first in order to use Nice Vibrations on
Unity.

Currently, all the steps below were tested using a macOS environment. Windows
steps are not provided at the moment ⛑️.

However, Windows support is available if iOS build support is not required. You
just need to skip the steps required for the macOS/iOS builds and edit the build
scripts according to your needs.

### Lofelt Studio SDK

To develop, build and run the Lofelt SDK there are the following requirements:

- Install [Rust](https://www.rust-lang.org/tools/install)
    - Make sure the `clippy` and `rustfmt` components are included, which is the
      default when installing via `rustup`
- Install
  [`rusty-hook`](https://github.com/swellaby/rusty-hook#optional-install)

    ```
    cargo install rusty-hook
    rusty-hook init
    ```
- macOS/iOS: Follow the steps in the [iOS README](./interfaces/ios/README.md)
- Android: Follow the steps in the [Android
  README](./interfaces/android/README.md)

The Rust core library itself does not generate haptics, for that the Android
library or iOS framework needs to be used. The Rust core library however has
unit tests that can be run, so it can make sense to work on it without any
interfaces.

### Nice Vibrations

To develop, build and run Nice Vibrations there are the following requirements:
- Make sure you can build the Unity Editor plugin host. Follow the steps in the
  [Unity Editor plugin host README](./unity-editor-plugin/README.md)
- Install Unity Hub and then  **Unity 2019.4.16f** as this is the lowest version
  Nice Vibrations supports.
- Install the Unity Input System package. It can be downloaded using the Unity
  Editor package manager.
- Install Doxygen CLI to generate the Nice Vibrations API documentation. On
  macOS, run:
  ```
  brew install doxygen
  ```

The source code of Nice Vibrations can be found in the
[interfaces/unity/NiceVibrations](./interfaces/unity/NiceVibrations/) Unity
project.


## Build and use Nice Vibrations Asset

Once the development environment is setup as specified in [Setting up the
Development Environment section](#setting-up-the-development-environment)
building the Nice Vibrations asset is possible by using one script:
```
sh build.sh
```

> ⚠️ This script will cross-compile Lofelt SDK for iOS and Android, but won't
cross-compile the [Unity Editor Plugin host](./unity-editor-plugin/README.md).
This means that if you're on macOS, the Unity Editor plugin host will only build
the shared library for macOS. To build for Windows, you need to run `sh
build.sh` in a Windows computer. Alternative you can use a release package
available in this page.

Running this script will take more than a couple of minutes, depending on your
machine. Meanwhile, have some coffee ☕ or tea 🍵.

Once its finished,a `nice-vibrations-asset` folder is created which contains the
NiceVibrations asset.

The NiceVibrations folder can then be imported into a Unity project:
   1. - Drag and drop the NiceVibrations folder into your project
   2. - Reimport the asset/restart Unity Editor
   3. - Install the [Unity Input
        System](https://docs.unity3d.com/Packages/com.unity.inputsystem@1.3/manual/Installation.html)
   4. - On the Project Settings > Player settings make sure to enable both Old
        and New input systems (make sure to follow all the steps in 3)

🎉 You're now ready to use Nice Vibrations 🎉.

Have a look at the Nice Vibrations API documentation, available in
[interfaces/unity/doxygen/html/index.html](./interfaces/unity/doxygen/html/index.html)
to know more.

If you would like to change the Nice Vibrations Unity asset code, check [this
section](./interfaces/unity/README#development) for more information.

## Build Lofelt Studio SDK individually and run tests

If you're not using Unity and just need the iOS Framework or Android library,
check the steps below.

To build for iOS (after doing the [setup for iOS](./interfaces/ios/README.md)):
```
sh build-platform.sh ios
```

To build for Android (after doing the [setup for
Android](./interfaces/android/README.md)):
```
sh build-platform.sh android
```

To build the SDK for iOS and Android for distribution:
```
sh release-sdk.sh
```

To build the lofelt-sdk Rust core library:
```
cargo build
```
The available binaries of the static library are located in `target/<debug> or
<release>/liblofelt_sdk.a`

To run the unit tests of the Rust core library:
```
cargo test
```

To build the Rust documentation:
```
cargo doc
```

To view the docs in your browser:
```
cargo doc --open
```

## Build and Test Lofelt SDK for iOS

The source code for iOS Framework is available in `interfaces/ios`. Check the
[README](./interfaces/ios/README.md) for more info.

## Build and Test Lofelt SDK for Android

The source code for Android library is available in `interfaces/android`. Check
the [README](./interfaces/android/README.md) for more info.

## Build and Test Lofelt Unity Editor plugin host

The source code for Unity Editor plugin is available in `unity-editor-plugin`.
Check the [README](./unity-editor-plugin/README.md) for more info.

## Development

An overview on how the code is structured can be seen in
[CONTRIBUTING.md](./CONTRIBUTING.md). If you would like to make changes, please
read it as it will be an easier starting to point.

## Git rules

We ensure code quality by using git hooks provided by
[`rusty-hooks`](https://github.com/swellaby/rusty-hook) for cargo. The git hook
setup can be seen in `.rusty-hook.toml` file.

# 🏗️ Continuous Integration

## GitHub actions

For CI, this repository uses Git Hub Actions (GHA). All the workflows are under
`.github/workflows` folder. In a nutshell, they setup the development
environment with the required dependencies and then run a `ci-<platform>.sh`
script with the appropriate steps we want to be valid.

The running strategy is as follows:
- On push: [rust-core.yml](.github/workflows/rust-core.yml)
- When merging to main: [ios.yml](.github/workflows/ios.yml),
  [android.yml](.github/workflows/android.yml) and
  [unity-editor-plugin.yml](.github/workflows/unity-editor-plugin.yml)

For releasing there are two workflows:
- **Release SDK and Unity workflow**: Install dependencies, run appropriate
  validation steps and then creates a GitHub release with both packages for the
  Lofelt Studio SDK and a Nice Vibrations. It is triggered once a tag is pushed
  under the `**sdk**nicevibrations**` format, e.g. `sdk-12.12.12-nicevibrations-10.10.1`.
- **Release haptic2ahap workflow**: Install dependencies and creates a GitHub
  release with a binary for the haptic2ahap CLI tool. It is triggered once a tag
  is pushed under the `**haptic2ahap**` format, e.g. `haptic2ahap-9.9.9`

## Locally

For local "sanity checks", the scripts that GHA runs can also be run locally.
Ideally they **should** be run before pushing code upstream to "fail fast".

These are all available under the `ci-<platform>.sh` pattern.

Currently. the `ci-unity.sh` does not run on GHA as this requires a Unity
license to be setup. Ideally, this script should be run before pushing code
upstream or releasing.

# ©️ License

Nice Vibrations is [MIT Licensed](LICENSE.md)

# ✍️ Authors

Nice Vibrations and Lofelt SDK were developed by Lofelt GmbH and the main
contributors were:
- [James Kneafsey](https://github.com/sesneaky)
- [João Freire](https://github.com/joaomcfreire)
- [Tomash Ghzegovsky](https://github.com/ghztomash)
- [Thomas Mcguire](https://github.com/tmcguire)
- [Ian Hobson](https://github.com/irh)
- [Renaud Forestié](https://github.com/reunono)
