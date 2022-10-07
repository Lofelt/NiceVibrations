// Copyright (c) Meta Platforms, Inc. and affiliates. 

using System;
using UnityEngine;
using System.Globalization;

namespace Lofelt.NiceVibrations
{
    /// <summary>
    /// A collection of methods to play simple haptic patterns.
    /// </summary>
    ///
    /// Each of the methods here load and play a simple haptic clip or a
    /// haptic pattern, depending on the device capabilities.
    ///
    /// None of the methods here are thread-safe and should only be called from
    /// the main (Unity) thread. Calling these methods from a secondary thread can
    /// cause undefined behaviour and memory leaks.
    ///
    /// After playback has finished, the loaded clips in this class will remain
    /// loaded in HapticController.

    public static class HapticPatterns
    {
        static String emphasisTemplate;
        static String constantTemplate;
        static NumberFormatInfo numberFormat;
        static private float[] constantPatternTime = new float[] { 0.0f, 0.0f };

        /// <summary>
        /// Enum that represents all the types of haptic presets available
        /// </summary>
        public enum PresetType
        {
            Selection = 0,
            Success = 1,
            Warning = 2,
            Failure = 3,
            LightImpact = 4,
            MediumImpact = 5,
            HeavyImpact = 6,
            RigidImpact = 7,
            SoftImpact = 8,
            None = -1
        }

        /// <summary>
        /// Structure that represents a haptic pattern with amplitude variations.
        /// </summary>
        ///
        /// \ref time values have be incremental to be compatible with Preset.
        struct Pattern
        {
            public float[] time;
            public float[] amplitude;

            static String clipJsonTemplate;

            static Pattern()
            {
                clipJsonTemplate = (Resources.Load("nv-pattern-template") as TextAsset).text;
            }

            public Pattern(float[] time, float[] amplitude)
            {
                this.time = time;
                this.amplitude = amplitude;
            }

            // Converts a Pattern to a GamepadRumble
            //
            // Each pair of adjacent entries in the Pattern create one entry in the GamepadRumble.
            public GamepadRumble ToRumble()
            {
                GamepadRumble result = new GamepadRumble();
                if (time.Length <= 1)
                {
                    return result;
                }

                Debug.Assert(time.Length == amplitude.Length);

                // The first pattern entry needs to have a time of 0.0 for the algorithm below to work
                Debug.Assert(time[0] == 0.0f);

                int rumbleCount = time.Length - 1;
                result.durationsMs = new int[rumbleCount];
                result.lowFrequencyMotorSpeeds = new float[rumbleCount];
                result.highFrequencyMotorSpeeds = new float[rumbleCount];
                result.totalDurationMs = 0;
                for (int rumbleIndex = 0; rumbleIndex < rumbleCount; rumbleIndex++)
                {
                    int patternDurationMs = (int)((time[rumbleIndex + 1] - time[rumbleIndex]) * 1000.0f);
                    result.durationsMs[rumbleIndex] = patternDurationMs;
                    result.lowFrequencyMotorSpeeds[rumbleIndex] = amplitude[rumbleIndex];
                    result.highFrequencyMotorSpeeds[rumbleIndex] = amplitude[rumbleIndex];
                    result.totalDurationMs += result.durationsMs[rumbleIndex];
                }
                return result;
            }

            // Converts a Pattern to a haptic clip JSON string.
            public String ToClip()
            {
                if (clipJsonTemplate == null)
                {
                    return "";
                }

                String amplitudeEnvelope = "";
                for (int i = 0; i < time.Length; i++)
                {
                    float clampedAmplitude = Mathf.Clamp(amplitude[i], 0.0f, 1.0f);
                    amplitudeEnvelope += "{ \"time\":" + time[i].ToString(numberFormat) + "," +
                                           "\"amplitude\":" + clampedAmplitude.ToString(numberFormat) + "}";

                    // Don't add a comma to the JSON data if we're at the end of the envelope
                    if (i + 1 < time.Length)
                    {
                        amplitudeEnvelope += ",";
                    }
                }

                return clipJsonTemplate.Replace("{amplitude-envelope}", amplitudeEnvelope);
            }
        }

        // A haptic preset in its different representations
        //
        // A Preset has four different representations, as there are four different playback methods.
        // Each representation is created at construction time, so that playing a
        // Preset has no further conversion cost at playback time.
        internal struct Preset
        {
            // For playback on iOS, using system haptics
            public PresetType type;

            // For playback on Android devices without amplitude control
            public float[] maximumAmplitudePattern;

            // For playback on Android devices with amplitude control
            public byte[] jsonClip;

            // For playback on gamepads
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            public GamepadRumble gamepadRumble;
#endif

            public Preset(PresetType type, float[] time, float[] amplitude)
            {
                Debug.Assert(type != PresetType.None);
                Pattern pattern = new Pattern(time, amplitude);
                this.type = type;
                this.maximumAmplitudePattern = pattern.time;
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
                this.gamepadRumble = pattern.ToRumble();
#endif
                this.jsonClip = System.Text.Encoding.UTF8.GetBytes(pattern.ToClip());
            }

            public float GetDuration()
            {
                if (maximumAmplitudePattern.Length > 0)
                {
                    return maximumAmplitudePattern[maximumAmplitudePattern.Length - 1];
                }
                else
                {
                    return 0f;
                }
            }
        }

        /// <summary>
        /// Predefined Preset that represents a "Selection" haptic preset
        /// </summary>
        internal static Preset Selection;

        /// <summary>
        /// Predefined Preset that represents a "Light" haptic preset
        /// </summary>
        internal static Preset Light;

        /// <summary>
        /// Predefined Preset that represents a "Medium" haptic preset
        /// </summary>
        internal static Preset Medium;

        /// <summary>
        /// Predefined Preset that represents a "Heavy" haptic preset
        /// </summary>
        internal static Preset Heavy;

        /// <summary>
        /// Predefined Preset that represents a "Rigid" haptic preset
        /// </summary>
        internal static Preset Rigid;

        /// <summary>
        /// Predefined Preset that represents a "Soft" haptic preset
        /// </summary>
        internal static Preset Soft;

        /// <summary>
        /// Predefined Preset that represents a "Success" haptic preset
        /// </summary>
        internal static Preset Success;

        /// <summary>
        /// Predefined Preset that represents a "Failure" haptic preset
        /// </summary>
        internal static Preset Failure;

        /// <summary>
        /// Predefined Preset that represents a "Warning" haptic preset
        /// </summary>
        internal static Preset Warning;

        static HapticPatterns()
        {
            emphasisTemplate = (Resources.Load("nv-emphasis-template") as TextAsset).text;
            constantTemplate = (Resources.Load("nv-constant-template") as TextAsset).text;

            numberFormat = new NumberFormatInfo();
            numberFormat.NumberDecimalSeparator = ".";

            // Initialize presets after setting the number format, so that the correct decimal
            // separator is used when building the JSON representation.

            Selection = new Preset(PresetType.Selection, new float[] { 0.0f, 0.04f },
                                                         new float[] { 0.471f, 0.471f });

            Light = new Preset(PresetType.LightImpact, new float[] { 0.000f, 0.040f },
                                                       new float[] { 0.156f, 0.156f });

            Medium = new Preset(PresetType.MediumImpact, new float[] { 0.000f, 0.080f },
                                                         new float[] { 0.471f, 0.471f });

            Heavy = new Preset(PresetType.HeavyImpact, new float[] { 0.0f, 0.16f },
                                                       new float[] { 1.0f, 1.00f });

            Rigid = new Preset(PresetType.RigidImpact, new float[] { 0.0f, 0.04f },
                                                       new float[] { 1.0f, 1.00f });

            Soft = new Preset(PresetType.SoftImpact, new float[] { 0.000f, 0.160f },
                                                     new float[] { 0.156f, 0.156f });

            Success = new Preset(PresetType.Success, new float[] { 0.0f, 0.040f, 0.080f, 0.240f },
                                                     new float[] { 0.0f, 0.157f, 0.000f, 1.000f });

            Failure = new Preset(PresetType.Failure,
                                 new float[] { 0.0f, 0.080f, 0.120f, 0.200f, 0.240f, 0.400f, 0.440f, 0.480f },
                                 new float[] { 0.0f, 0.470f, 0.000f, 0.470f, 0.000f, 1.000f, 0.000f, 0.157f });

            Warning = new Preset(PresetType.Warning, new float[] { 0.0f, 0.120f, 0.240f, 0.280f },
                                                     new float[] { 0.0f, 1.000f, 0.000f, 0.470f });
        }

        /// <summary>
        /// Plays a single emphasis point.
        /// </summary>
        ///
        /// Plays a haptic clip that consists only of one breakpoint with emphasis.
        /// On iOS, this translates to a transient, and on Android and gamepads to
        /// a quick vibration.
        ///
        /// <param name="amplitude">The amplitude of the emphasis, from 0.0 to 1.0</param>
        /// <param name="frequency">The frequency of the emphasis, from 0.0 to 1.0</param>
        public static void PlayEmphasis(float amplitude, float frequency)
        {
            if (emphasisTemplate == null || !HapticController.hapticsEnabled)
            {
                return;
            }

            // Use HapticController.Play() to play a .haptic clip on mobile devices
            // that support it, or to play a gamepad rumble if a gamepad is connected.
            if (HapticController.Init() || GamepadRumbler.IsConnected())
            {
                float clampedAmplitude = Mathf.Clamp(amplitude, 0.0f, 1.0f);
                float clampedFrequency = Mathf.Clamp(frequency, 0.0f, 1.0f);
                const float duration = 0.1f;

                String json = emphasisTemplate
                    .Replace("{amplitude}", clampedAmplitude.ToString(numberFormat))
                    .Replace("{frequency}", clampedFrequency.ToString(numberFormat))
                    .Replace("{duration}", duration.ToString(numberFormat));

                // This preprocessor section will only run for non-mobile platforms
                GamepadRumble rumble = new GamepadRumble();
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
                rumble.durationsMs = new int[] { (int)(duration * 1000) };
                rumble.lowFrequencyMotorSpeeds = new float[] { clampedAmplitude };
                rumble.highFrequencyMotorSpeeds = new float[] { clampedFrequency };
#endif

                HapticController.Load(System.Text.Encoding.UTF8.GetBytes(json), rumble);
                HapticController.Loop(false);
                HapticController.Play();
            }

            // As a fallback, play a short buzz on Android, or a preset on iOS.
            else if (DeviceCapabilities.isVersionSupported)
            {
#if (UNITY_ANDROID && !UNITY_EDITOR)
                LofeltHaptics.PlayMaximumAmplitudePattern(new float[]{ 0.0f, 0.05f });
#elif (UNITY_IOS && !UNITY_EDITOR)
                PresetType preset = presetTypeForEmphasis(amplitude);
                LofeltHaptics.TriggerPresetHaptics((int)preset);
#endif
            }
        }

        /// <summary>
        /// Automatically selects the fallback preset based on the emphasis point amplitude.
        /// </summary>
        ///
        /// <param name="amplitude">The amplitude of the emphasis, from 0.0 to 1.0</param>
        static PresetType presetTypeForEmphasis(float amplitude)
        {
            if (amplitude > 0.5f)
            {
                return HapticPatterns.PresetType.HeavyImpact;
            }
            else if (amplitude <= 0.5f && amplitude > 0.3)
            {
                return HapticPatterns.PresetType.MediumImpact;
            }
            else
            {
                return HapticPatterns.PresetType.LightImpact;
            }
        }

        /// <summary>
        /// Plays a haptic with constant amplitude and frequency.
        /// </summary>
        ///
        /// On iOS and with gamepads, you can use HapticController::clipLevel to modulate the haptic
        /// while it is playing. iOS additional supports modulating the frequency with
        /// HapticController::clipFrequencyShift.
        ///
        /// When \ref DeviceCapabilities.meetsAdvancedRequirements returns false on mobile,
        /// the behavior of this method is different for iOS and Android:
        /// <ul>
        ///     <li>On iOS, it will play the preset <c>HapticPatterns.PresetType.HeavyImpact</c>. </li>
        ///
        ///     <li>On Android, it will play a pattern with maximum amplitude for the set <c>duration</c>
        ///      since there is no amplitude control.</li>
        ///
        /// </ul>
        /// <param name="amplitude">Amplitude, from 0.0 to 1.0</param>
        /// <param name="frequency">Frequency, from 0.0 to 1.0</param>
        /// <param name="duration">Play duration in seconds</param>
        public static void PlayConstant(float amplitude, float frequency, float duration)
        {
            if (constantTemplate == null || !HapticController.hapticsEnabled)
            {
                return;
            }

            float clampedAmplitude = Mathf.Clamp(amplitude, 0.0f, 1.0f);
            float clampedFrequency = Mathf.Clamp(frequency, 0.0f, 1.0f);
            float clampedDurationSecs = Mathf.Max(duration, 0.0f);

            String json = constantTemplate
                .Replace("{duration}", clampedDurationSecs.ToString(numberFormat));

            // This preprocessor section will only run for non-mobile platforms
            GamepadRumble rumble = new GamepadRumble();
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            int rumbleDurationMs = (int)(clampedDurationSecs * 1000);
            const int rumbleEntryDurationMs = 16; // One rumble entry per frame at 60 FPS, which is the limit of what GamepadRumbler can play
            int rumbleEntryCount = rumbleDurationMs / rumbleEntryDurationMs;
            rumble.durationsMs = new int[rumbleEntryCount];
            rumble.lowFrequencyMotorSpeeds = new float[rumbleEntryCount];
            rumble.highFrequencyMotorSpeeds = new float[rumbleEntryCount];

            // Create many rumble entries instead of just one. With just one entry, changing
            // clipLevel while the rumble is playing would have no effect, as GamepadRumbler applies
            // a change only to the next rumble entry, not the one currently playing.
            for (int i = 0; i < rumbleEntryCount; i++)
            {
                rumble.durationsMs[i] = rumbleEntryDurationMs;
                rumble.lowFrequencyMotorSpeeds[i] = 1.0f;
                rumble.highFrequencyMotorSpeeds[i] = 1.0f;
            }
#endif

            if (HapticController.Init() || GamepadRumbler.IsConnected())
            {
                HapticController.Load(System.Text.Encoding.UTF8.GetBytes(json), rumble);
                HapticController.Loop(false);
                HapticController.clipLevel = clampedAmplitude;
                HapticController.clipFrequencyShift = clampedFrequency;
                HapticController.Play();
            }
            else if (DeviceCapabilities.isVersionSupported)
            {
#if (UNITY_ANDROID && !UNITY_EDITOR)
                constantPatternTime[1] = duration;
                LofeltHaptics.PlayMaximumAmplitudePattern(constantPatternTime);
#elif (UNITY_IOS && !UNITY_EDITOR)
                HapticPatterns.PlayPreset(PresetType.HeavyImpact);
#endif
            }
        }

        static Preset GetPresetForType(PresetType type)
        {
            Debug.Assert(type != PresetType.None);

            switch (type)
            {
                case PresetType.Selection:
                    return Selection;
                case PresetType.LightImpact:
                    return Light;
                case PresetType.MediumImpact:
                    return Medium;
                case PresetType.HeavyImpact:
                    return Heavy;
                case PresetType.RigidImpact:
                    return Rigid;
                case PresetType.SoftImpact:
                    return Soft;
                case PresetType.Success:
                    return Success;
                case PresetType.Failure:
                    return Failure;
                case PresetType.Warning:
                    return Warning;
            }

            // Silence compiler warning about not all code paths returning something
            return Medium;
        }

        /// <summary>
        /// Plays a set of predefined haptic patterns.
        /// </summary>
        ///
        /// These predefined haptic patterns are played and represented in different ways for iOS,
        /// Android and gamepads.
        ///
        /// - On iOS, this function triggers system haptics that are native to iOS. Calling
        ///   \ref HapticController.Stop() won't stop haptics.
        /// - On Android devices that can play <c>.haptic</c> clips (DeviceCapabilities.meetsAdvancedRequirements
        ///   is <c>true</c>) and on gamepads, this function plays a haptic pattern that has a similar
        ///   experience to the matching iOS system haptics.
        /// - On Android devices that can not play <c>.haptic</c> clips (DeviceCapabilities.meetsAdvancedRequirements
        ///   is <c>false</c>), this function plays a haptic pattern that has a similar experience to
        ///   the matching iOS system haptics, by turning the motor off and on at maximum amplitude.
        ///
        /// This is a "fire-and-forget" method. Other functionalities like seeking, looping, and
        /// runtime modulation won't work after calling this method.
        ///
        /// <param name="presetType">Type of preset represented by a \ref PresetType enum</param>
        public static void PlayPreset(PresetType presetType)
        {
            if (!HapticController.hapticsEnabled || presetType == PresetType.None)
            {
                return;
            }

            Preset preset = GetPresetForType(presetType);

#if (UNITY_IOS && !UNITY_EDITOR)
            LofeltHaptics.TriggerPresetHaptics((int)presetType);
            return;
#else
            if (HapticController.Init() || GamepadRumbler.IsConnected())
            {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
                HapticController.Load(preset.jsonClip, preset.gamepadRumble);
#else
                HapticController.Load(preset.jsonClip);
#endif
                HapticController.Loop(false);
                HapticController.Play();
                return;
            }

            if (DeviceCapabilities.isVersionSupported)
            {
#if (UNITY_ANDROID && !UNITY_EDITOR)
                LofeltHaptics.PlayMaximumAmplitudePattern(preset.maximumAmplitudePattern);
                return;
#endif
            }
#endif
        }

        /// <summary>
        /// Returns the haptic preset duration.
        /// </summary>
        ///
        /// While a preset is played back in different ways on iOS, Android and gamepads, the
        /// duration is similar for each playback method.
        ///
        /// <param name="presetType"> Type of preset represented by a \ref PresetType enum </param>
        /// <returns>Returns a float with a the preset duration; if the selected preset is `None`, it returns 0</returns>
        public static float GetPresetDuration(PresetType presetType)
        {
            if (presetType == PresetType.None)
            {
                return 0;
            }

            return GetPresetForType(presetType).GetDuration();
        }
    }

}
