![](https://github.com/Lofelt/lofelt-sdk/workflows/ios/badge.svg)
![](https://github.com/Lofelt/lofelt-sdk/workflows/android/badge.svg)
![](https://github.com/Lofelt/lofelt-sdk/workflows/rust-core/badge.svg)

# Contents

- [Contents](#contents)
- [Lofelt SDK](#lofelt-sdk)
- [Getting Started](#getting-started)
  - [Setting up the Development Environment](#setting-up-the-development-environment)
  - [Build library and run tests](#build-library-and-run-tests)
  - [Build and Test Lofelt SDK for iOS](#build-and-test-lofelt-sdk-for-ios)
  - [Build and Test Lofelt SDK for Android](#build-and-test-lofelt-sdk-for-android)
  - [Build and Test Lofelt Unity Plugin](#build-and-test-lofelt-unity-plugin)
  - [Git rules](#git-rules)
- [Structure](#structure)

# Lofelt SDK
The Lofelt SDK including core library, interfaces and clip-players

# Getting Started

## Setting up the Development Environment

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

## Build library and run tests

To build for iOS (after doing the [setup for iOS](./interfaces/ios/README.md)):
```
sh build.sh ios
```

To build for Android (after doing the [setup for Android](./interfaces/android/README.md)):
```
sh build.sh android
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

## Build and Test Lofelt Unity Plugin

The source code for Unity Editor plugin is available in `unity-editor-plugin`. Check the [README](./unity-editor-plugin/README.md) for more info.

## Git rules

We ensure code quality by using git hooks provided by [`rusty-hooks`](https://github.com/swellaby/rusty-hook) for cargo. The git hook setup can be seen in `.rusty-hook.toml` file.

# Structure

- [interfaces](./interfaces/README.md) provides native frameworks for the SDK.
- [apps](./apps/README.md) contains applications using the features SDK, using the native frameworks from [Interfaces](./interfaces/README.md).
- [examples](./examples/README.md) contains example code and snippets for applications using SDK features.
- [licenses](./licenses/) contains the licence files to be included in the release package.
- [core](./core/README.md) routes haptic data and commands to clip-players
- [clip-players](./clip-players/README.md) Playback of pre-authored clips with various different implementations.
- [unity-editor-plugin](./unity-editor-plugin/README.md) A plugin for the Unity editor to convert haptic clips
