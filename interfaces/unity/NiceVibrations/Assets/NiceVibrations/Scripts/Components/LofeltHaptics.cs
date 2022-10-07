// Copyright (c) Meta Platforms, Inc. and affiliates. 

using UnityEngine;
using System;

#if (UNITY_ANDROID && !UNITY_EDITOR)
using System.Text;
using System.Runtime.InteropServices;
#elif (UNITY_IOS && !UNITY_EDITOR)
using UnityEngine.iOS;
using System.Runtime.InteropServices;
#endif

namespace Lofelt.NiceVibrations
{
    /// <summary>
    /// C# wrapper for the Lofelt Studio Android and iOS SDK.
    /// </summary>
    ///
    /// You should not use this class directly, use HapticController instead, or the
    /// <c>MonoBehaviour</c> classes HapticReceiver and HapticSource.
    ///
    /// The Lofelt Studio Android and iOS SDK are included in Nice Vibrations as pre-compiled
    /// binary plugins.
    ///
    /// Each method here delegates to either the Android or iOS SDK. The methods should only be
    /// called if DeviceMeetsMinimumPlatformRequirements() returns true, otherwise there will
    /// be runtime errors.
    ///
    /// All the methods do nothing when running in the Unity editor.
    ///
    /// Before calling any other method, Initialize() needs to be called.
    ///
    /// Errors are printed and swallowed, no exceptions are thrown. On iOS, this happens inside
    /// the SDK, on Android this happens with try/catch blocks in this class and in JNIHelpers.
    public static class LofeltHaptics
    {
#if (UNITY_ANDROID && !UNITY_EDITOR)
        static AndroidJavaObject lofeltHaptics;
        static AndroidJavaObject hapticPatterns;
        static long nativeController;

        // Cache the most commonly used JNI method IDs during initialization.
        // Calling a Java method via its method ID is faster and uses less allocations than
        // calling a method by string, like e.g. 'lofeltHaptics.Call("play")'.
        static IntPtr playMethodId = IntPtr.Zero;
        static IntPtr stopMethodId = IntPtr.Zero;
        static IntPtr seekMethodId = IntPtr.Zero;
        static IntPtr loopMethodId = IntPtr.Zero;
        static IntPtr setAmplitudeMultiplicationMethodId = IntPtr.Zero;
        static IntPtr playMaximumAmplitudePattern = IntPtr.Zero;

        [DllImport("lofelt_sdk")]
        private static extern bool lofeltHapticsLoadDirect(IntPtr controller, [In] byte[] bytes, long size);

#elif (UNITY_IOS && !UNITY_EDITOR)
        // imports of iOS Framework bindings

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsDeviceMeetsMinimumRequirementsBinding();

        [DllImport("__Internal")]
        private static extern IntPtr lofeltHapticsInitBinding();

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsLoadBinding(IntPtr controller, [In] byte[] bytes, long size);

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsPlayBinding(IntPtr controller);

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsStopBinding(IntPtr controller);

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsSeekBinding(IntPtr controller, float time);

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsSetAmplitudeMultiplicationBinding(IntPtr controller, float factor);

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsSetFrequencyShiftBinding(IntPtr controller, float shift);

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsLoopBinding(IntPtr controller, bool enable);

        [DllImport("__Internal")]
        private static extern float lofeltHapticsGetClipDurationBinding(IntPtr controller);

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsReleaseBinding(IntPtr controller);

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsSystemHapticsTriggerBinding(int type);

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsSystemHapticsInitializeBinding();

        [DllImport("__Internal")]
        private static extern bool lofeltHapticsSystemHapticsReleaseBinding();

        static IntPtr controller = IntPtr.Zero;

        static bool systemHapticsInitialized = false;
#endif

        /// <summary>
        /// Initializes the iOS framework or Android library plugin.
        /// </summary>
        ///
        /// This needs to be called before calling any other method.
        public static void Initialize()
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            try
            {
                using (var unityPlayerClass = new AndroidJavaClass("com.unity3d.player.UnityPlayer"))
                using (var context = unityPlayerClass.GetStatic<AndroidJavaObject>("currentActivity"))
                {
                    lofeltHaptics = new AndroidJavaObject("com.lofelt.haptics.LofeltHaptics", context);
                    nativeController = lofeltHaptics.Call<long>("getControllerHandle");
                    hapticPatterns = new AndroidJavaObject("com.lofelt.haptics.HapticPatterns", context);

                    playMethodId = AndroidJNIHelper.GetMethodID(lofeltHaptics.GetRawClass(), "play", "()V", false);
                    stopMethodId = AndroidJNIHelper.GetMethodID(lofeltHaptics.GetRawClass(), "stop", "()V", false);
                    seekMethodId = AndroidJNIHelper.GetMethodID(lofeltHaptics.GetRawClass(), "seek", "(F)V", false);
                    loopMethodId = AndroidJNIHelper.GetMethodID(lofeltHaptics.GetRawClass(), "loop", "(Z)V", false);
                    setAmplitudeMultiplicationMethodId = AndroidJNIHelper.GetMethodID(lofeltHaptics.GetRawClass(), "setAmplitudeMultiplication", "(F)V", false);
                    playMaximumAmplitudePattern = AndroidJNIHelper.GetMethodID(hapticPatterns.GetRawClass(), "playMaximumAmplitudePattern", "([F)V", false);
                }
            }
            catch (Exception ex)
            {
                Debug.LogException(ex);
            }
#elif (UNITY_IOS && !UNITY_EDITOR)
            lofeltHapticsSystemHapticsInitializeBinding();
            systemHapticsInitialized = true;
            controller = lofeltHapticsInitBinding();
#endif
        }

        /// <summary>
        /// Releases the resources used by the iOS framework or Android library plugin.
        /// </summary>
        public static void Release()
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            try
            {
                lofeltHaptics.Dispose();
                lofeltHaptics = null;

                hapticPatterns.Dispose();
                hapticPatterns = null;
            }
            catch (Exception ex)
            {
                Debug.LogWarning(ex);
            }
#elif (UNITY_IOS && !UNITY_EDITOR)
            if(DeviceCapabilities.isVersionSupported) {
                lofeltHapticsSystemHapticsReleaseBinding();
                if(controller != IntPtr.Zero) {
                    lofeltHapticsReleaseBinding(controller);
                    controller = IntPtr.Zero;
                }
            }
#endif
        }

        public static bool DeviceMeetsMinimumPlatformRequirements()
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            return JNIHelpers.Call<bool>(lofeltHaptics, "deviceMeetsMinimumRequirements");
#elif (UNITY_IOS && !UNITY_EDITOR)
            return lofeltHapticsDeviceMeetsMinimumRequirementsBinding();
#else
            return true;
#endif
        }

        public static void Load(byte[] data)
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            // For performance reasons, we do *not* call into the Java API with
            // `lofeltHaptics.Call("load", data)` here. Instead, we bypass the Java layer and
            // call into the native library directly, saving the costly conversion from
            // C#'s byte[] to Java's byte[].
            //
            // No exception handling needed here, lofeltHapticsLoadDirect() is a native method that
            // doesn't throw an exception and instead logs the error.
            lofeltHapticsLoadDirect((IntPtr)nativeController, data, data.Length);
#elif (UNITY_IOS && !UNITY_EDITOR)
            lofeltHapticsLoadBinding(controller, data, data.Length);
#endif
        }

        public static float GetClipDuration()
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            return JNIHelpers.Call<float>(lofeltHaptics, "getClipDuration");
#elif (UNITY_IOS && !UNITY_EDITOR)
            return lofeltHapticsGetClipDurationBinding(controller);
#else
            //No haptic clip was loaded with Lofelt SDK, so it returns 0.0f
            return 0.0f;
#endif
        }

        public static void Play()
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            JNIHelpers.Call(lofeltHaptics, playMethodId);
#elif (UNITY_IOS && !UNITY_EDITOR)
            lofeltHapticsPlayBinding(controller);
#endif
        }

        public static void PlayMaximumAmplitudePattern(float[] timings)
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            JNIHelpers.Call(hapticPatterns, playMaximumAmplitudePattern, timings);
#endif
        }

        public static void Stop()
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            JNIHelpers.Call(lofeltHaptics, stopMethodId);
#elif (UNITY_IOS && !UNITY_EDITOR)
            lofeltHapticsStopBinding(controller);
#endif
        }

        public static void StopPattern()
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            try
            {
                hapticPatterns.Call("stopPattern");
            }
            catch (Exception ex)
            {
                Debug.LogWarning(ex);
            }
#endif
        }

        public static void Seek(float time)
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            JNIHelpers.Call(lofeltHaptics, seekMethodId, time);
#elif (UNITY_IOS && !UNITY_EDITOR)
            lofeltHapticsSeekBinding(controller, time);
#endif
        }

        public static void SetAmplitudeMultiplication(float factor)
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            JNIHelpers.Call(lofeltHaptics, setAmplitudeMultiplicationMethodId, factor);
#elif (UNITY_IOS && !UNITY_EDITOR)
            lofeltHapticsSetAmplitudeMultiplicationBinding(controller, factor);
#endif
        }

        public static void SetFrequencyShift(float shift)
        {
#if (UNITY_IOS && !UNITY_EDITOR)
            lofeltHapticsSetFrequencyShiftBinding(controller, shift);
#endif
        }

        public static void Loop(bool enabled)
        {
#if (UNITY_ANDROID && !UNITY_EDITOR)
            JNIHelpers.Call(lofeltHaptics, loopMethodId, enabled);
#elif (UNITY_IOS && !UNITY_EDITOR)
            lofeltHapticsLoopBinding(controller, enabled);
#endif
        }

        public static void TriggerPresetHaptics(int type)
        {
#if (UNITY_IOS && !UNITY_EDITOR)
            if (!systemHapticsInitialized)
            {
                lofeltHapticsSystemHapticsInitializeBinding();
                systemHapticsInitialized = true;
            }
            lofeltHapticsSystemHapticsTriggerBinding(type);
#endif
        }
    }
}
