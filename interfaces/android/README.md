# Contents
- [Contents](#contents)
- [Lofelt SDK for Android](#lofelt-sdk-for-android)
- [Structure](#structure)
- [Setting up the Development Environment](#setting-up-the-development-environment)
- [Building](#building)
  - [From Command Line](#from-command-line)
    - [Android Library](#android-library)
    - [Rust Library](#rust-library)
  - [From Android Studio](#from-android-studio)
  - [Troubleshooting](#troubleshooting)
- [Running automated tests](#running-automated-tests)
  - [From Command Line](#from-command-line-1)
  - [From Android Studio](#from-android-studio-1)
- [Building the example app](#building-the-example-app)
- [Caveats and issues](#caveats-and-issues)

# Lofelt SDK for Android

This folder has the Android library project containing source code for the Lofelt SDK for Android.

# Structure

- `LofeltHaptics/` folder contains the gradle project with source files and tests of the Android library
  - `LofeltHaptics/` contains the library module. This is where the source code of the library and the tests is.
    - `src/main/java/` contains the Java source code for the library
    - `src/androidTest/java/` contains the Java source code for the instrumented test
    - `build.gradle` contains the main part of the build configuration, including the plugin that builds the Rust JNI library
  - `build-library.sh` contains a script that builds the Android library in release mode
  - `build/` contains the build artifacts. The Android library gets put in `outputs/aar/`.

# Setting up the Development Environment

- Install the latest Android Studio version with Standard settings.
- In Android Studio, install the SDK for the API level we're compiling against (currently API level
  30, see `compileSdkVersion` in `interfaces/android/LofeltHaptics/LofeltHaptics/build.gradle`).
  SDKs can be installed with *Tools->SDK Manager->SDK Platforms*. If you want to debug into the
  Java code of the Android SDK, also install the sources here.
- In Android Studio, install the NDK. The NDK can be installed with *Tools->SDK Manager->SDK Tools*.
Currently, the Lofelt SDK requires having NDK version `21.3.6528147` installed.

- If you want to run the instrumented test in the simulator, create a simulator by using
  *Tools->AVD Manager* in Android Studio.
- Install the Rust targets to be able to compile for Android ARM and x86 Android devices
  ```
  rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
  ```
- While not necessary for building the library, it can be useful to have the `adb` tool in the
  `PATH`. On macOS:
  ```
  export PATH=~/Library/Android/sdk/platform-tools:$PATH
  ```
- Setup Android Studio to automatically format Java files when saving
  - Install the "Save Actions" plugin
  - In *Preferences->Other Settings->Save Actions*, a few things need to be checked:
    - Activate Save Actions on save
    - Optimize imports
    - Reformat file

# Building

All the instructions below assume that the current working directory is the
`LofeltHaptics/` project folder.

## From Command Line
### Android Library

To build the Android library in release mode, just run:
```
./build-library.sh
```

You can also build by invoking gradle directly, which can be useful for building in debug mode
or when needing a more verbose build:
```
./gradlew assembleDebug -info
```

### Rust Library
If you want to build just the Rust core library for Android, run:
```
./gradlew cargoBuild
```

If you want to build the Rust core library for a specific target, you can invoke
`cargo` manually. First build in verbose mode to see the exact command invoked
by gradle:
```
./gradlew cargoBuild -info
```
This should print something along the lines of:
```
> Task :LofeltHaptics:cargoBuildX86_64
...
Starting process 'command 'cargo''.
Command: cargo build --verbose --release --target=x86_64-linux-android

```
You can then invoke this `cargo` command manually, after changing into the correct working
directory.

## From Android Studio
- In Android Studio, open the `LofeltHaptics` project by opening the `LofeltHaptics/` folder.
- In the *Project* sidebar, select the LofeltHaptics module (not the project)
- Chose the desired build type (debug or release) with *Build->Select Build Variant...*
- Build with *Build->Make Module LofeltHaptics.LofeltHaptics*

## Troubleshooting

If you got a similar error to the one below:
```
(..)
Please check that /Library/Internet Plug-Ins/JavaAppletPlugin.plugin/Contents/Home contains a valid JDK installation.
```

Make sure you have `JAVA_HOME` set as an environment variable. Depending on your macOS version it should located at */Library/Java/JavaVirtualMachines/<jdk_folder>/Contents/Home/*

# Running automated tests

The project does not contain unit tests, as unit tests can not load JNI libraries. This is because
JNI libraries are compiled for the target device, not for the host.
Instead the project contains instrumented tests that need to be run on a device or simulator.

## From Command Line
- Make sure you have either a real device or a simulator device connected, by running
  `adb devices`.
  - A simulator can be started with the AVD Manager in Android Studio
  - A device can be connected via USB and [enabling USB debugging](https://developer.android.com/studio/debug/dev-options#enable).
- Run `./gradlew connectedAndroidTest`

Alternatively run the tests from the lofelt-sdk root
- Attach a device
- `sh test-on-device.sh android`

## From Android Studio
See the [Android developer documentation on tests](https://developer.android.com/studio/test#run_a_test).

# Building the example app

The example app references `android-library/LofeltHaptics.aar`. That file is created by `build-platform.sh`,
so in order to build the example app, the build script needs to be run first:
```
./build-platform.sh android
```

It is also possible to create a project that includes both the example app and the library with its
sources. This is useful when making changes in the library that need to be tested in the example
app. With such a project that can be done without manually switching between projects.
The following changes are needed:
- `settings.gradle`
    ```
    +include ':LofeltHaptics'
    +project(":LofeltHaptics").projectDir = file("../../../interfaces/android/LofeltHaptics/LofeltHaptics")
    include ':app'
    ```
- `build.gradle` (app-level)
    ```
        implementation 'androidx.appcompat:appcompat:1.2.0'
        implementation 'com.google.android.material:material:1.2.1'
        implementation 'androidx.constraintlayout:constraintlayout:2.0.1'
    -    implementation files('../../../../android-library/LofeltHaptics.aar')
    +    implementation project(path: ':LofeltHaptics')
    ```
- `build.gradle` (top-level)
    ```
        repositories {
            google()
            jcenter()
    +        maven {
    +            url "https://plugins.gradle.org/m2/"
    +        }
        }
        dependencies {
            classpath "com.android.tools.build:gradle:4.1.0-rc02"
    +        classpath 'gradle.plugin.org.mozilla.rust-android-gradle:plugin:0.8.3'
    ```


# Caveats and issues
- **Changes to the Rust library are only included in the Android library when building twice**.
  Building twice is done by the build script `build-library.sh`, but in Android Studio you need
  to do this manually yourself.
- Debugging the native Rust code is not yet possible.
- `stdout` (`println!()`, `dbg!()`) and `stderr` (`eprintln!()`) are not forwarded
  to the Android logging system and are not visible anywhere. Use the `log` crate instead when
  doing print-style debugging.
- Cleaning a build does not clean the Rust libraries. You need to do this manually with
  `cargo clean`. See https://github.com/mozilla/rust-android-gradle/issues/42.
- The Android library is not code signed. On Android this is possible but not required.
- The Android library is not minified/obfuscated. Since that would only affect the Java code
  and not the Rust core library, it would not gain us much.
- The Rust library contains some code that isn't needed on Android, such as AHAP
