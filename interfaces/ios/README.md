# Contents

- [Contents](#contents)
- [Lofelt SDK for iOS](#lofelt-sdk-for-ios)
- [Structure](#structure)
- [Setting up the Development Environment](#setting-up-the-development-environment)
- [Building](#building)
  - [Setting up the Xcode project for building](#setting-up-the-xcode-project-for-building)
  - [Building the framework](#building-the-framework)
  - [Building the framework for a single target](#building-the-framework-for-a-single-target)
  - [Building the SDK library manually](#building-the-sdk-library-manually)
- [Including the framework in an app](#including-the-framework-in-an-app)
  - [Including the framework binary in an iOS app](#including-the-framework-binary-in-an-ios-app)
  - [Including the framework source in an iOS app](#including-the-framework-source-in-an-ios-app)
- [Running automated device tests](#running-automated-device-tests)
  - [From Command Line](#from-command-line)
  - [From Xcode](#from-xcode)


# Lofelt SDK for iOS

This folder has the iOS Framework project containing source code for the Lofelt SDK for iOS.

# Structure

- `LofeltHaptics` folder contains the Xcode project with source files and tests of the iOS Frameworks.
    - `LofeltHaptics` contains the source and header files of the iOS Framework, and is where the Public API for iOS is defined.
    - `LofeltHapticsTestAppTests` contains integration tests that can only run on iPhones (they can't run on a simulator)
    - `LofeltHapticsTestApp` host app to run LofeltHapticsTestAppTests on iPhones.
- `build-xcframework.sh` is a script to build pre-compiled framework binaries for all iOS platforms. This allows integration of our framework without users having access to the source code. It does not need to be used for development and can be a way for building and distributing the framework without the use of CocoaPods.

# Setting up the Development Environment

- ⚠️ Make sure you have set Rust's 1.55 toolchain as the default toolchain. Currently, building for iOS requires Rust 1.55 toolchain.

- Install cargo-lipo:

   `cargo install cargo-lipo`

  You don't have to run `cargo lipo` directly as this happens automatically via a script in the Xcode project for Lofelt SDK for iOS. If you ever want to do it manually the steps are under "Building the library manually" below

- Add a rust compiler that can compile for iOS targets:

  `rustup target add aarch64-apple-ios x86_64-apple-ios`

- Add the nightly-2021-10-05 toolchain:

  Download from https://github.com/getditto/rust-bitcode/releases/tag/nightly-2021-10-05 and follow the [installation instructions](https://github.com/getditto/rust-bitcode#pre-compiled-releases).

  > ℹ️ This is required when building the framework with bitcode enabled, which is needed when running the `build-xcframework.sh` script.

- Install Xcode 13 or later from the App Store.
  > ℹ️ The latest version used for these steps was Xcode 13.4.

- Update your iPhone or simulator to iOS 13.0 or later

- Allow to run the Rust toolchain in the Gatekeeper settings

  The first time you build for iOS you'll eventually get macOS Gatekeeper warnings like "'cargo' cannot be opened because the developer cannot be verified". To allow to run these programs, go to System Preferences > Security and Privacy, then click the button to allow the program that was blocked and restart the build. This step needs to be repeated multiple times.

# Building

## Setting up the Xcode project for building

This should be already done but if you hit errors while building you might need to repeat these steps.

Open the Xcode project for the SDK iOS framework:

`open interfaces/ios/LofeltHaptics/LofeltHaptics.xcodeproj/`

With the Xcode project open
- Click on Project in tree
- Make sure project, not target is selected in settings
- Click Build Settings
- Filter for All and Combined
- Enter "bitcode" in the search box
- On the "Enable Bitcode" row (not the "Debug" or "Release" row), select 'No'

## Building the framework

We have a script to build the framework:

`build-xcframework.sh`.

It must be run from its own directory.

`cd interfaces/ios/LofeltHaptics/`
`sh build-xcframework.sh`

After running this script you will have two new folders under `interfaces/ios/LofeltHaptics/build`:

- `XCArchive` - containing archived `ios` (`devices`) and `simulator` images.
- `XCFramework` - containing combined image (formerly `universal`).

XCArchive contains the `devices` and `simulator` builds separately and is not intended for distribution. The XCFramework folder contains the single, multi-platform library recommended for distribution.

## Building the framework for a single target

We prefer to use the build script above but if you have a reason to build for a single target this is how you do it.

Choose LofeltHaptics as the active scheme (to the right of the stop button in Xcode)

Run `Product -> Build` (⌘ B)

You now have a built SDK iOS framework

## Building the SDK library manually

Again we prefer to use the build script above but if you have a reason to build the SDK library manually for iOS these are the steps.

Run cargo-lipo

`cargo lipo`

Confirm you get a built SDK for iOS

`ls -la target/universal/debug/`

You should see:

`liblofelt_sdk.a`

# Including the framework in an app

## Including the framework binary in an iOS app

As mentioned in "Building the framework" above there are three frameworks that result from our build script. An app developer would use each of these for different things:

- `ios.xcarchive` - for testing on devices only and for releasing to Apple's AppStore.
- `ios_simulator.xcarchive` - for testing on simulator only. No haptics can be experience though. Cannot be included in an app released on the AppStore
- `LofeltHaptics.xcframework` - for testing on devices and the simulator, the target architecture is automatically selected by the build system. Many developers might use this one during development, and it is the new recommended method for distributing binary libraries.

Whichever of these is chosen, copy the entire .xcframework or .framework folder into the project in which you want to include it.

## Including the framework source in an iOS app

- Close the Xcode instance that has the LofeltHaptics framework project open
- Open the iOS app project with Xcode
- Click on Project in tree
- Make sure project, not target is selected in settings
- Under Build Phases, expand "Link Binaries With Libraries"
- Click +
- Click "Add other"
- Browse to: NiceVibrations/interfaces/ios/LofeltHaptics/LofeltHaptics.xcodeproj

Now the framework project shows up under Frameworks in the Xcode Project Navigator

- Now expand the framework project
- Expand Products
- Drag the `.xcframework` file onto "Link Binaries With Libraries" under Build Phases

Before building, you need to choose the right configuration (release vs debug, device vs simulator,
bitcode enabled vs disabled). Be aware of the following restrictions:
- Building for the simulator with bitcode enabled will not work, as a bitcode-enabled build will use
  the `ios-arm64` toolchain when building the Rust core, which doesn't support the simulator target (`x86_64-apple-ios`)
- Building in debug mode with bitcode enabled will not work due to
  https://github.com/rust-lang/rust/issues/67223

Now you can debug your project and the framework together.

# Running automated device tests

## From Command Line
- Make sure you have an iPhone connected
- Run `xcrun xctrace list devices` to get the name of your iPhone
- `cd "interfaces/ios/LofeltHaptics"`
- `sh tests-run-on-device.sh "My iPhone"`

Alternatively run the tests from the repository's root
- Run `xcrun xctrace list devices` to get the name of your iPhone
- `sh test-on-device.sh ios "My iPhone"`

## From Xcode
- Select `LofeltHapticsTestApp` as the active scheme (dropdown to the right of the stop button in Xcode)
- Select iPhone as the target device
- In `LofeltHapticsTestAppTests.swift` you can select if you want to run Performance Tests, by setting `enablePerformanceTests = true`. By default, they don't run.
- From menu choose Product > Test or press ⌘U
- Xcode will notify you if tests succeed or fail
- You can also see test results in the [Test Navigator](https://developer.apple.com/library/archive/documentation/DeveloperTools/Conceptual/testing_with_xcode/chapters/05-running_tests.html)
