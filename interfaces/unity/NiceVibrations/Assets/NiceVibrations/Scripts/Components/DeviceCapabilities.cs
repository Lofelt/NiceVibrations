// Copyright (c) Meta Platforms, Inc. and affiliates. 

using UnityEngine;

#if (UNITY_IOS && !UNITY_EDITOR)
    using UnityEngine.iOS;
#endif

namespace Lofelt.NiceVibrations
{
    /// <summary>
    /// A class containing properties that describe the current device capabilities for use with
    /// Nice Vibrations
    /// </summary>
    ///
    /// This class describes the capabilities of an iOS or Android device, gamepads are not handled
    /// by it.
    public static class DeviceCapabilities
    {
        /// <summary>
        /// Property that holds the current RuntimePlatform
        /// </summary>
        public static RuntimePlatform platform { get; }

        /// <summary>
        /// Property that holds the current platform version.
        /// </summary>
        /// iOS version on iOS, Android API level on Android or 0 otherwise.
        public static int platformVersion { get; }

        /// <summary>
        /// Indicates if the device meets the requirements to play advanced haptics with
        /// Nice Vibrations
        /// </summary>
        ///
        /// Advanced requirements means that the device can play back <c>.haptic</c> clips.
        /// While devices that don't meet the advanced requirements can not play back <c>.haptic</c>
        /// clips, they can still play back simpler fallback haptics as long as
        /// \ref isVersionSupported is <c>true</c>.
        ///
        /// While DeviceCapabilities.isVersionSupported only checks the OS version, this method
        /// additionally checks the device capabilities.
        ///
        /// The required device capabilities are:
        /// - iOS: iPhone >= 8
        /// - Android: Amplitude control for the <c>Vibrator</c>
        ///
        /// You don't usually need to check this property. All other methods in HapticController
        /// will check \ref meetsAdvancedRequirements before calling into <c>LofeltHaptics</c>.
        /// In case the device does not support advanced haptics there is a possibility of fallback
        /// haptics based on presets.
        public static bool meetsAdvancedRequirements
        {
            get
            {
                return _meetsAdvancedRequirements;
            }
        }
        private static bool _meetsAdvancedRequirements;

        /// <summary>
        /// Indicates if the OS version is high enough to play haptics with Nice Vibrations.
        /// </summary>
        ///
        /// The minimum required versions are:
        /// - iOS >= 11
        /// - Android API level >= 17
        ///
        /// This only checks the minimum supported OS version in terms of API and does not guarantee
        /// that advanced haptics with amplitude control can be recreated, For that check with
        /// \ref meetsAdvancedRequirements.
        public static bool isVersionSupported { get; }

        /// <summary>
        /// Indicates if the device is capable of amplitude control in order to recreate
        /// advanced haptics.
        /// </summary>
        public static bool hasAmplitudeControl
        {
            get
            {
                return _hasAmplitudeControl;
            }
        }
        private static bool _hasAmplitudeControl;

        /// <summary>
        /// Indicates if the device is capable of changing the frequency of haptic signals
        /// </summary>
        public static bool hasFrequencyControl
        {
            get
            {
                return _hasFrequencyControl;
            }
        }
        private static bool _hasFrequencyControl;

        /// <summary>
        /// Indicates if the device is capable of real-time amplitude modulation of haptic signals
        /// </summary>
        public static bool hasAmplitudeModulation
        {
            get
            {
                return _hasAmplitudeModulation;
            }
        }
        private static bool _hasAmplitudeModulation;

        /// <summary>
        /// Indicates if the device is capable of real-time frequency modulation of haptic signals
        /// </summary>
        public static bool hasFrequencyModulation
        {
            get
            {
                return _hasFrequencyModulation;
            }
        }
        private static bool _hasFrequencyModulation;

        /// <summary>
        /// Indicates if the device is capable of natively reproducing emphasized haptics
        /// </summary>
        public static bool hasEmphasis
        {
            get
            {
                return _hasEmphasis;
            }
        }
        private static bool _hasEmphasis;

        /// <summary>
        /// Indicates if the device is capable of emulating emphasized haptics
        /// </summary>
        public static bool canEmulateEmphasis
        {
            get
            {
                return _canEmulateEmphasis;
            }
        }
        private static bool _canEmulateEmphasis;

        /// <summary>
        /// Indicates if the device is capable of looping haptic clips
        /// </summary>
        public static bool canLoop
        {
            get
            {
                return _canLoop;
            }
        }
        private static bool _canLoop;

        /// <summary>
        /// Constructor that fills in the only the DeviceCapabilities platform version properties.
        /// </summary>
        /// This is separate of Init() because we need to first check the version numbers before
        /// initializing <c>LofeltHaptics</c>
        static DeviceCapabilities()
        {
            platform = Application.platform;
            platformVersion = 0;
            isVersionSupported = false;

#if (UNITY_ANDROID && !UNITY_EDITOR)
            platformVersion = int.Parse(SystemInfo.operatingSystem.Substring(SystemInfo.operatingSystem.IndexOf("-") + 1, 3));
            const int minimumSupportedAndroidSDKVersion = 17;
            isVersionSupported = platformVersion >= minimumSupportedAndroidSDKVersion;
#elif (UNITY_IOS && !UNITY_EDITOR)
            string versionString = Device.systemVersion;
            string[] versionArray = versionString.Split('.');
            platformVersion = int.Parse(versionArray[0]);
            const int minimumSupportedIOSVersion = 11;
            isVersionSupported = platformVersion >= minimumSupportedIOSVersion;

            DeviceGeneration generation = Device.generation;
            if ((generation == DeviceGeneration.iPhone3G)
            || (generation == DeviceGeneration.iPhone3GS)
            || (generation == DeviceGeneration.iPodTouch1Gen)
            || (generation == DeviceGeneration.iPodTouch2Gen)
            || (generation == DeviceGeneration.iPodTouch3Gen)
            || (generation == DeviceGeneration.iPodTouch4Gen)
            || (generation == DeviceGeneration.iPhone4)
            || (generation == DeviceGeneration.iPhone4S)
            || (generation == DeviceGeneration.iPhone5)
            || (generation == DeviceGeneration.iPhone5C)
            || (generation == DeviceGeneration.iPhone5S)
            || (generation == DeviceGeneration.iPhone6)
            || (generation == DeviceGeneration.iPhone6Plus)
            || (generation == DeviceGeneration.iPhone6S)
            || (generation == DeviceGeneration.iPhone6SPlus)
            || (generation == DeviceGeneration.iPhoneSE1Gen)
            || (generation == DeviceGeneration.iPad1Gen)
            || (generation == DeviceGeneration.iPad2Gen)
            || (generation == DeviceGeneration.iPad3Gen)
            || (generation == DeviceGeneration.iPad4Gen)
            || (generation == DeviceGeneration.iPad5Gen)
            || (generation == DeviceGeneration.iPadAir1)
            || (generation == DeviceGeneration.iPadAir2)
            || (generation == DeviceGeneration.iPadMini1Gen)
            || (generation == DeviceGeneration.iPadMini2Gen)
            || (generation == DeviceGeneration.iPadMini3Gen)
            || (generation == DeviceGeneration.iPadMini4Gen)
            || (generation == DeviceGeneration.iPadPro10Inch1Gen)
            || (generation == DeviceGeneration.iPadPro10Inch2Gen)
            || (generation == DeviceGeneration.iPadPro11Inch)
            || (generation == DeviceGeneration.iPadPro1Gen)
            || (generation == DeviceGeneration.iPadPro2Gen)
            || (generation == DeviceGeneration.iPadPro3Gen)
            || (generation == DeviceGeneration.iPadUnknown)
            || (generation == DeviceGeneration.iPodTouch1Gen)
            || (generation == DeviceGeneration.iPodTouch2Gen)
            || (generation == DeviceGeneration.iPodTouch3Gen)
            || (generation == DeviceGeneration.iPodTouch4Gen)
            || (generation == DeviceGeneration.iPodTouch5Gen)
            || (generation == DeviceGeneration.iPodTouch6Gen)
            || (generation == DeviceGeneration.iPhone6SPlus))
            {
                isVersionSupported = false;
            }

#elif (UNITY_EDITOR)
            isVersionSupported = true;
#endif
        }

        /// <summary>
        /// Function that initializes the rest of the DeviceCapabilities properties.
        /// Must be called after <c>LofeltHaptics</c> was initialized.
        /// </summary>
        public static void Init()
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            _hasAmplitudeControl = LofeltHaptics.DeviceMeetsMinimumPlatformRequirements();
            _canEmulateEmphasis = LofeltHaptics.DeviceMeetsMinimumPlatformRequirements();
            _canLoop = LofeltHaptics.DeviceMeetsMinimumPlatformRequirements();
#elif (UNITY_IOS && !UNITY_EDITOR)
            _hasAmplitudeControl = LofeltHaptics.DeviceMeetsMinimumPlatformRequirements();
            _hasFrequencyControl = LofeltHaptics.DeviceMeetsMinimumPlatformRequirements();
            _hasAmplitudeModulation = LofeltHaptics.DeviceMeetsMinimumPlatformRequirements();
            _hasFrequencyModulation = LofeltHaptics.DeviceMeetsMinimumPlatformRequirements();
            _hasEmphasis = LofeltHaptics.DeviceMeetsMinimumPlatformRequirements();
            _canLoop = LofeltHaptics.DeviceMeetsMinimumPlatformRequirements();
#endif
            _meetsAdvancedRequirements = LofeltHaptics.DeviceMeetsMinimumPlatformRequirements();
        }
    }
}
