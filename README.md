![](https://github.com/Lofelt/NiceVibrations/workflows/ios/badge.svg)
![](https://github.com/Lofelt/NiceVibrations/workflows/android/badge.svg)
![](https://github.com/Lofelt/NiceVibrations/workflows/rust-core/badge.svg)

# Contents

- [Contents](#contents)
- [What is Nice Vibrations?](#what-is-nice-vibrations)
  - [Main features](#main-features)
- [Documentation](#documentation)
- [Getting Started](#getting-started)
  - [Setting up the Development Environment on macOS](#setting-up-the-development-environment-on-macos)
    - [Lofelt Studio SDK](#lofelt-studio-sdk)
    - [Nice Vibrations](#nice-vibrations)
  - [Build and use Nice Vibrations Asset](#build-and-use-nice-vibrations-asset)
  - [Build Lofelt Studio SDK individually and run tests](#build-lofelt-studio-sdk-individually-and-run-tests)
  - [Build and Test Lofelt SDK for iOS](#build-and-test-lofelt-sdk-for-ios)
  - [Build and Test Lofelt SDK for Android](#build-and-test-lofelt-sdk-for-android)
  - [Build and Test Lofelt Unity Editor plugin host](#build-and-test-lofelt-unity-editor-plugin-host)
  - [Git rules](#git-rules)

# What is Nice Vibrations?

Built on top of Lofelt Studio SDK, Nice Vibrations offers a universal interface to trigger HD haptic
feedback on supported Unity platforms platforms at once, as well as ways to get exactly the right
vibration on each platform.

Nice Vibrations provides full support for Android and iOS haptic APIs (including iOS Core Haptics)
as well as gamepad rumble.

Nice Vibrations now comes with a growing collection of royalty-free HD haptic (and audio) assets
ready for quick prototyping and testing. No design skills or tasks required. The assets cover
various use cases like Application UX, Game FX, Game Objects, and more.

This repository contains both the source of Lofelt Studio SDK and Nice Vibrations.

## Main features
- Play and Stop: play and stop complex HD haptic clips (.haptic files).
- iOS improved performance: load and play haptic clips 2x faster than on Nice Vibrations 3.9
- Loop: loop a playing clip.
- Seek: jump into any point of your haptic clip before or during playback depending on what is
happening in your game.
- Modulate Frequency or Amplitude: adjust the strength or frequency of your playing haptic clip in
  reaction to other on screen events (distance between characters, size of a collision), or just to
  create variation in the pattern playback (repetitive actions like weapon firing or footsteps and
  jumps).
- Haptic Voice Priority: set the priority of your haptic clip. Lower priority will be silenced if
  higher priority clips are triggered at the same time.
- Code Only API: no need to be stuck in Unity and tied to MonoBehaviours.
  With the code-only API, you can work in any development environment.
- MonoBehaviour API: play haptics by just using the Unity Editor, without writing a single line of
  C# code.
- Platform Independence: use the same C# and the same .haptic file on all platforms, no need to
  write platform-specific code for Android or iOS.
- Haptic Library: no need to design your own haptics. A growing library of clips is included to get
  you started quickly and with limited experience.
- Global Haptic Level and Mute: disable haptics entirely with one switch or reduce the intensity of
  haptics across the whole application.
- Fully Featured Demos: demos included to help you learn how to apply all of the features for your
  specific use case. These demos will also help you see how to migrate your existing Nice Vibrations
  code to the new API’s.
- Haptic Presets Support: trigger 9 predefined haptic patterns with a single line of code.
- Gamepad Rumble: play .haptic files on gamepads supported by Unity.
- Fallback Support: play basic haptics on older phones and operating systems.
- And more..

# Documentation

The [Nice Vibrations documentation](https://lofelt.github.io/integrating-haptics/nice-vibrations-by-lofelt/)
explains how to integrate and use Nice Vibrations with new or existing Unity projects.

# Getting Started

## Setting up the Development Environment on macOS

Nice Vibrations depends on the cross-platform Lofelt Studio SDK which means that the Lofelt Studio
SDK needs to be build first in order to use Nice Vibrations on Unity.

Currently, all the steps below were tested using a macOS environment. Windows steps are not provided at
the moment.

However, Windows support is available if iOS support is not required. You just need to skip the steps
required for the macOS/iOS builds and edit the build scripts according to your needs.

### Lofelt Studio SDK

To develop, build and run the Lofelt SDK there are the following requirements:

- Install [Rust](https://www.rust-lang.org/tools/install)
    - Make sure the `clippy` and `rustfmt` components are included, which is the default when installing via `rustup`
- Install [`rusty-hook`](https://github.com/swellaby/rusty-hook#optional-install)

    ```
    cargo install rusty-hook
    rusty-hook init
    ```
- macOS/iOS: Follow the steps in the [iOS README](./interfaces/ios/README.md)
- Android: Follow the steps in the [Android README](./interfaces/android/README.md)

The Rust core library itself does not generate haptics, for that the Android library
or iOS framework needs to be used. The Rust core library however has unit tests that
can be run, so it can make sense to work on it without any interfaces.

### Nice Vibrations

To develop, build and run Nice Vibrations, the Unity Editor version required is at least **Unity
2019.4.16f or newer**. The source code of Nice Vibrations can be found in the
[interfaces/unity/NiceVibrations](./interfaces/unity/NiceVibrations/) Unity project.

In addition, Nice Vibrations depends on the Unity Input System package which
can be downloaded using the Unity Editor package manager.

## Build and use Nice Vibrations Asset

Once the development environment is setup as specified in [Setting up the Development Environment section](#setting-up-the-development-environment)
it is possible to build the Nice Vibrations asset with one script:
```
sh build.sh
```

> ⚠️ This script will cross-compile Lofelt SDK for iOS and Android, but won't cross-compile the [Unity
Editor Plugin host](./unity-editor-plugin/README.md). This means that if you're on macOS, the Unity Editor plugin host will only build
the shared library for macOS. To build for Windows, you need to run `sh build.sh` in a Windows computer.
Alternative you can use a release package available in this page.

Running this script will take more than a couple of minutes, depending on your machine.
Meanwhile, have some coffee ☕ or tea 🍵 .

Once its finished,a `nice-vibrations-asset` folder is created which contains the NiceVibrations asset.

The NiceVibrations folder can then be imported into a Unity project:
   1. - Drag and drop the NiceVibrations folder into your project
   2. - Reimport the asset/restart Unity Editor
   3. - Install the [Unity Input System](https://docs.unity3d.com/Packages/com.unity.inputsystem@1.3/manual/Installation.html)
   4. - On the Project Settings > Player settings make sure to enable both Old and New input systems (make sure to follow all the steps in 3)

You're now ready to use Nice Vibrations. Have a look at the [documentation](#documentation) to know more.

## Build Lofelt Studio SDK individually and run tests

If you're not using Unity and just need the iOS Framework or Android library, check the steps below.

To build for iOS (after doing the [setup for iOS](./interfaces/ios/README.md)):
```
sh build-sdk.sh ios
```

To build for Android (after doing the [setup for Android](./interfaces/android/README.md)):
```
sh build-sdk.sh android
```

To build the SDK for iOS and Android for distribution:
```
sh release-sdk.sh
```

To build the lofelt-sdk Rust core library:
```
cargo build
```
The available binaries of the static library are located in `target/<debug> or <release>/liblofelt_sdk.a`


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

The source code for iOS Framework is available in `interfaces/ios`. Check the [README](./interfaces/ios/README.md) for more info.

## Build and Test Lofelt SDK for Android

The source code for Android library is available in `interfaces/android`. Check the [README](./interfaces/android/README.md) for more info.

## Build and Test Lofelt Unity Editor plugin host

The source code for Unity Editor plugin is available in `unity-editor-plugin`. Check the [README](./unity-editor-plugin/README.md) for more info.

## Git rules

We ensure code quality by using git hooks provided by [`rusty-hooks`](https://github.com/swellaby/rusty-hook) for cargo. The git hook setup can be seen in `.rusty-hook.toml` file.
