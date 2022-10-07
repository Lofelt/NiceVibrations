# Nice Vibrations Release Notes

## Change Log

### Nice Vibrations for Unity `4.1.2`

**7 October 2022**

##### 🚀 **Improvements**
- Fixed an output level issue on gamepads. The output level is now set after loading a clip into the GamepadRumbler.
- Fixed builds when building for tvOS. Nice Vibrations doesn't support tvOS but it was breaking the build. Now, this problem is fixed.
- When using Nice Vibrations on iPhones, sometimes haptics didn't play. And also, there was a big memory leak on iPhones. This iOS specific issues are now fixed.

### Nice Vibrations for Unity `4.1.1`

**14 April 2022**

#### 🚀 **Improvements**
- Fixes an issue where haptic clips would not import for gamepads on Apple Silicon builds of Unity.
- Fixes an issue where on Android, on some locales, PlayPreset() would fail with an error.
- Fixes an issue where Nice Vibrations didn't work at all for customers using ProGuard or R8 on Android.

### Nice Vibrations for Unity `4.1.0`

**9 February 2022**

##### ✨ **New features**

- Added the new "Objects" haptic samples pack by Lofelt, with 40 new haptic clips crafted with gameplay in mind.

##### 🚀 **Improvements**

- The Unity editor plugin on macOS is now supported on Apple Silicon architectures. This fixes an issue
where it wasn't possible to play rumble on Gamepads when running on Macbooks with M1, M1 Max, and M1
Pro CPUs.
- Previously, some Android devices were detecting a gamepad connected even though they didn't have any
gamepad connected. This is due to how some Android devices capabilities are interpreted by Unity's
Android player. Since Unity Input System currently doesn't support rumble on mobile for gamepads,
we disabled runtime checks for gamepads on mobile for now to avoid this problem.
- Improved performance by reducing some memory allocations.
- Android phones with basic haptic capabilities can now call `PlayConstant()`. This allows  a constant vibration to be triggered for a specified amount of time.
- Improved haptic preset playback on Android phones with basic capabilities. Previously, some presets
couldn't be felt on some devices.

##### ⚠️ **Known issues**

- AHAP support is currently only available when using the 3.9 plugin that's included in the asset. 

### Nice Vibrations `4.0.1`

**18 November 2021**

##### 🚀 **Improvements**

- Fixed a string localization issue on iOS.
- Calling `HapticPatterns.Play()` before calling `HapticController.Init()`, was raising an exception on Android. This is now fixed.
- The build process was optimized and the post-build script was removed. The issue of not having the NiceVibrations folder in the Assets folder of the Unity project is now solved.
- Fixed text alignment issues on the Emphasized haptics tab of the Demo scene.
- Removed console warning of an unused variable when the gamepad rumble feature is disabled or when the playback requirements aren't met.

##### ⚠️ **Known issues**

- AHAP support is currently only available when using the 3.9 plugin that's included in the asset. 

### Nice Vibrations `4.0.0`

**25 October 2021**

##### ✨ **New features**

- [API documentation is now live!](/nice-vibrations-api-docs/index.html)
- Haptic presets are now available! Choose from 9 simple haptic presets that can easily be added
from code.
- Added fallback support for older devices. Devices that can't play HD haptics can now fallback
for haptic presets of the developer choice.
- New capabilities check API, which allows you to check at runtime the capabilities of the device.
This enables developers to make better decisions when some haptic capabilities are not available.
- Nice Vibrations 4 now allows you to play .haptic files in Gamepads and more.

##### 🚀 **Improvements**

- Improved haptic clip loading times on iOS and Android
- Replaced `SetAmplitudeMultiplication` method with `clipLevel` and `outputLevel` properties.
- Replaced `SetFrequencyShift` method with a `clipFrequencyShift` property.
- Removed dependency of Nice Vibrations 3.9 on the demo scenes.
- Improved setting clip level/output level to produce a linear haptic change.
- Demo scene didn't have sound for the Continuous and Emphasis tabs. That's now fixed.

##### ⚠️ **Known issues**

- AHAP support is currently only available when using the 3.9 plugin that's included in the asset. 

### Nice Vibrations `4.0.0-beta.1`

**13 August 2021**

These are the changes [since Nice Vibrations 3.9](https://nice-vibrations.moremountains.com/nice-vibrations-releases)
##### ✨ **New features**

- Core functionality using new native libraries based on [Lofelt Studio SDK](https://github.com/Lofelt/NiceVibrations);
- Playback of haptic clips (.haptic files) with a universal haptic format that allows you to play haptics on supported platforms (iOS and Android for now);
- Loop haptic clips;
- Seek functionality that lets you jump into any point of your haptic clip before or during playback;
- Adjust the strength or pitch of haptic clips by changing amplitude and frequency at runtime;
- Set the playback priority of your haptic clip. Lower priority will be silenced if higher priority clips are triggered at the same time;
- Adds platform independence, so that you can use the same C# and the same .haptic file on all platforms, no need to write platform-specific code for Android or iOS;
- Updated Code-only API that allows you to achieve platform independence;
- Added MonoBehaviour API that allows you to play haptics by just using the Unity Editor, without writing C# scripts;
- New Haptic Library with a growing library of +80 clips is included to get you started quickly and with limited experience;
- Updated demos with the newly update Code-only API;
- Wobble tab demo updated with MonoBehaviour API;
- Added Android emphasis emulation allowing your haptic clips with emphasis points to have an improved experience on Android devices;
- Include Asset Package of version 3.9 to allow you to use the older plugin if you need fallback, AHAP or Gamepad rumble playback support.

##### ️⚠️ **Known issues**

- AHAP support is currently only available when using the 3.9 plugin that's included in the asset. 

