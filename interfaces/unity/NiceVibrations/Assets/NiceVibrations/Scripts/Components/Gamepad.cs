// Copyright (c) Meta Platforms, Inc. and affiliates. 

using System;
using System.Diagnostics;
using System.Timers;
using UnityEngine;

// There are 3 conditions for working gamepad support in Nice Vibrations:
//
// 1. NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED - The input system package needs to be installed.
//    See https://docs.unity3d.com/Packages/com.unity.inputsystem@1.0/manual/Installation.html#installing-the-package
//    This is set by Nice Vibrations' assembly definition file, using a version define.
//    See https://docs.unity3d.com/Manual/ScriptCompilationAssemblyDefinitionFiles.html#define-symbols
//    about version defines, and see Lofelt.NiceVibrations.asmdef for the usage in Nice Vibrations.
//
// 2. ENABLE_INPUT_SYSTEM - The input system needs to be enabled in the project settings.
//    See https://docs.unity3d.com/Packages/com.unity.inputsystem@1.0/manual/Installation.html#enabling-the-new-input-backends
//    This define is set by Unity, see https://docs.unity3d.com/Manual/PlatformDependentCompilation.html
//
// 3. NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT - This is a user-defined define which needs to be not set.
//    NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT is not set by default. It can be set by a user in the
//    player settings to disable gamepad support completely. One reason to do this is to reduce the
//    size of a HapticClip asset, as setting this define changes to HapticImporter to not add the
//    GamepadRumble to the HapticClip. Changing this define requires re-importing all .haptic clip
//    assets to update HapticClip's GamepadRumble.
//
// If any of the 3 conditions is not met, GamepadRumbler doesn't contain any calls into
// UnityEngine.InputSystem, and CanPlay() always returns false.
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
using UnityEngine.InputSystem;
#endif

namespace Lofelt.NiceVibrations
{
    /// <summary>
    /// Contains a vibration pattern to make a gamepad rumble.
    /// </summary>
    ///
    /// GamepadRumble contains the information on when to set what motor speeds on a gamepad
    /// to make it rumble with a specific pattern.
    ///
    /// GamepadRumble has three arrays of the same length representing the rumble pattern. The
    /// entries for each array index describe for how long to turn on the gamepad's vibration
    /// motors, at what speed.
    [Serializable]
    public struct GamepadRumble
    {
        /// <summary>
        /// The duration, in milliseconds, that the motors will be turned on at the speed set
        /// in \ref lowFrequencyMotorSpeeds and \ref highFrequencyMotorSpeeds at the same array
        /// index
        /// </summary>
        [SerializeField]
        public int[] durationsMs;

        /// <summary>
        /// The total duration of the GamepadRumble, in milliseconds
        /// </summary>
        [SerializeField]
        public int totalDurationMs;

        /// <summary>
        /// The motor speeds of the low frequency motor
        /// </summary>
        [SerializeField]
        public float[] lowFrequencyMotorSpeeds;

        /// <summary>
        /// The motor speeds of the high frequency motor
        /// </summary>
        [SerializeField]
        public float[] highFrequencyMotorSpeeds;

        /// <summary>
        /// Checks if the GamepadRumble is valid and also not empty
        /// </summary>
        /// <returns>Whether the GamepadRumble is valid</returns>
        public bool IsValid()
        {
            return durationsMs != null &&
                   lowFrequencyMotorSpeeds != null &&
                   highFrequencyMotorSpeeds != null &&
                   durationsMs.Length == lowFrequencyMotorSpeeds.Length &&
                   durationsMs.Length == highFrequencyMotorSpeeds.Length &&
                   durationsMs.Length > 0;
        }
    }

    /// <summary>
    /// Vibrates a gamepad based on a GamepadRumble rumble pattern.
    /// </summary>
    ///
    /// GamepadRumbler can load and play back a GamepadRumble pattern on the current
    /// gamepad.
    ///
    /// This is a low-level class that normally doesn't need to be used directly. Instead,
    /// you can use HapticSource and HapticController to play back haptic clips, as those
    /// classes support gamepads by using GamepadRumbler internally.
    public static class GamepadRumbler
    {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
        static GamepadRumble loadedRumble;

        static bool rumbleLoaded = false;

        // This Timer is used to wait until it is time to advance to the next entry in loadedRumble.
        // When the Timer is elapsed, ProcessNextRumble() is called to set new motor speeds to the
        // gamepad.
        static Timer rumbleTimer = new Timer();

        // The index of the entry of loadedRumble that is currently being played back
        static int rumbleIndex = -1;

        // The total duration of rumble entries that have been played back so far
        static long rumblePositionMs = 0;

        // Keeps track of how much time elapsed since playback was started
        static Stopwatch playbackWatch = new Stopwatch();

        /// <summary>
        /// A multiplication factor applied to the motor speeds of the low frequency motor.
        /// </summary>
        ///
        /// The multiplication factor is applied to the low frequency motor speed of every
        /// GamepadRumble entry before playing it.
        ///
        /// In other words, this applies a gain (for factors greater than 1.0) or an attenuation
        /// (for factors less than 1.0) to the clip. If the resulting speed of an entry is
        /// greater than 1.0, it is clipped to 1.0. The speed is clipped hard, no limiter is
        /// used.
        ///
        /// The motor speed multiplication is reset when calling Load(), so Load() needs to be
        /// called first before setting the multiplication.
        ///
        /// A change of the multiplication is applied to a currently playing rumble, but only
        /// for the next rumble entry, not the one currently playing.
        public static float lowFrequencyMotorSpeedMultiplication = 1.0f;

        /// <summary>
        /// Same as \ref lowFrequencyMotorSpeedMultiplication, but for the high frequency speed
        /// motor.
        /// </summary>
        public static float highFrequencyMotorSpeedMultiplication = 1.0f;

        static int currentGamepadID = -1;

#endif

        /// <summary>
        /// Initializes the GamepadRumbler.
        /// </summary>
        ///
        /// This needs to be called from the main thread, which is the reason why this is a method
        /// instead of a static constructor: Sometimes Unity calls static constructors from a
        /// different thread, and an explicit Init() method gives us more control over this.
        public static void Init()
        {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            // Initialize rumbleTimer, so that ProcessNextRumble() will be called on the main thread
            // when the timer is triggered.
            var syncContext = System.Threading.SynchronizationContext.Current;
            rumbleTimer.Elapsed += (object obj, System.Timers.ElapsedEventArgs args) =>
            {
                syncContext.Post(_ =>
                {
                    ProcessNextRumble();
                }, null);
            };
#endif
        }

        /// <summary>
        /// Checks whether a call to Play() would trigger playback on a gamepad.
        /// </summary>
        ///
        /// Playing back a rumble pattern with Play() only works if a gamepad is connected and if
        /// a GamepadRumble has been loaded with Load() before.
        ///
        /// <returns>Whether a vibration can be triggered on a gamepad</returns>
        public static bool CanPlay()
        {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            return IsConnected() && rumbleLoaded && loadedRumble.IsValid();
#else
            return false;
#endif
        }

#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
        /// <summary>
        /// Gets the Gamepad object corresponding to the specified gamepad ID.
        /// </summary>
        ///
        /// If the specified ID is out of range of the connected gamepad(s),
        /// <c>InputSystem.Gamepad.current</c> will be returned.
        ///
        /// <param name="gamepadID">The ID of the gamepad to be returned.</c> </param>
        /// <returns> A <c> InputSystem.Gamepad</c> </returns>
        static UnityEngine.InputSystem.Gamepad GetGamepad(int gamepadID)
        {
            if (gamepadID >= 0)
            {
                if (gamepadID >= UnityEngine.InputSystem.Gamepad.all.Count)
                {
                    return UnityEngine.InputSystem.Gamepad.current;
                }
                else
                {
                    return UnityEngine.InputSystem.Gamepad.all[gamepadID];
                }
            }
            return UnityEngine.InputSystem.Gamepad.current;
        }
#endif

        /// <summary>
        /// Set the current gamepad for haptics playback by ID.
        /// </summary>
        ///
        /// This method needs be called before haptics playback, e.g. \ref HapticController.Play(),
        /// \ref HapticPatterns.PlayEmphasis(), \ref HapticPatterns.PlayConstant(), etc, for
        /// for the gamepad to be properly selected.
        ///
        /// If this method isn't called, haptics will be played on <c>InputSystem.Gamepad.current</c>
        ///
        /// For example, if you have 3 controllers connected, you have to choose between values 0, 1,
        /// and 2.
        ///
        /// If the gamepad ID value doesn't match any connected gamepad, calling
        /// this method has no effect.
        /// <param name="gamepadID">The ID of the gamepad</param>
        public static void SetCurrentGamepad(int gamepadID)
        {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            if (gamepadID < UnityEngine.InputSystem.Gamepad.all.Count)
            {
                currentGamepadID = gamepadID;
            }
#endif
        }

        /// <summary>
        /// Checks whether a gamepad is connected and recognized by Unity's input system.
        /// </summary>
        ///
        /// If the input system package is not installed or not enabled, the gamepad is not
        /// recognized and treated as not connected here.
        ///
        /// If the <c>NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT</c> define is set in the player settings,
        /// this function pretends no gamepad is connected.
        ///
        /// <returns>Whether a gamepad is connected</returns>
        public static bool IsConnected()
        {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            return GetGamepad(currentGamepadID) != null;
#else
            return false;
#endif
        }

        /// <summary>
        /// Loads a rumble pattern for later playback.
        /// </summary>
        ///
        /// <param name="rumble">The rumble pattern to load</param>
        public static void Load(GamepadRumble rumble)
        {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            if (rumble.IsValid())
            {
                loadedRumble = rumble;
                rumbleLoaded = true;
                lowFrequencyMotorSpeedMultiplication = 1.0f;
                highFrequencyMotorSpeedMultiplication = 1.0f;
            }
            else
            {
                Unload();
            }
#endif
        }

        /// <summary>
        /// Plays back the rumble pattern loaded previously with Load().
        /// </summary>
        ///
        /// If no rumble pattern has been loaded, or if no gamepad is connected, this method does
        /// nothing.
        public static void Play()
        {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            if (CanPlay())
            {
                rumbleIndex = 0;
                rumblePositionMs = 0;
                playbackWatch.Restart();
                ProcessNextRumble();
            }
#endif
        }

        /// <summary>
        /// Stops playback previously started with Play() by turning off the gamepad's motors.
        /// </summary>
        public static void Stop()
        {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            if (GetGamepad(currentGamepadID) != null)
            {
                GetGamepad(currentGamepadID).ResetHaptics();
            }
            rumbleTimer.Enabled = false;
            rumbleIndex = -1;
            rumblePositionMs = 0;
            playbackWatch.Stop();
#endif
        }

        /// <summary>
        /// Stops playback and unloads the currently loaded GamepadRumble from memory.
        /// </summary>
        public static void Unload()
        {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            loadedRumble.highFrequencyMotorSpeeds = null;
            loadedRumble.lowFrequencyMotorSpeeds = null;
            loadedRumble.durationsMs = null;
            rumbleLoaded = false;
            Stop();
#endif
        }

        // Advances the position in the GamepadRumble by one.
        //
        // If the end of the rumble has been reached, playback is stopped and false is returned.
        private static bool IncreaseRumbleIndex()
        {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            rumblePositionMs += loadedRumble.durationsMs[rumbleIndex];
            rumbleIndex++;
            if (rumbleIndex == loadedRumble.durationsMs.Length)
            {
                Stop();
                return false;
            }

            return true;
#else
            return false;
#endif
        }

        // Processes the next entry in loadedRumble by setting the gamepad's motor speeds to the
        // speeds stored in that entry.
        //
        // Afterwards, the rumbleTimer is set to call this method again, after the time stored
        // in entry of loadedRumble.
        private static void ProcessNextRumble()
        {
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            // rumbleIndex can be -1 after Stop() has been called after the call to
            // ProcessNextRumble() has already been queued up via SynchronizationContext.
            if (rumbleIndex == -1)
            {
                return;
            }

            if (rumbleIndex == loadedRumble.durationsMs.Length)
            {
                Stop();
                return;
            }

            UnityEngine.Debug.Assert(loadedRumble.IsValid());
            UnityEngine.Debug.Assert(rumbleLoaded);
            UnityEngine.Debug.Assert(rumbleIndex >= 0 && rumbleIndex <= loadedRumble.durationsMs.Length);

            // Figure out for how long the current rumble entry should be played (durationToWait).
            // Due to the timer not waiting for exactly the same amount of time that we requested,
            // there can be a bit of error that we need to compensate for. For example, if the timer
            // waited for 3ms longer than we requested, we play the next rumble entry for a 3ms
            // less to compensate for that.
            // In fact, Unity triggers the timer only once per frame, so at 30 FPS, the timer
            // resolution is 32ms. That means that the timing error can be bigger than the duration
            // of the whole rumble entry, and to compensate for that, the entire rumble entry needs
            // to be skipped. That's what the loop does: It skips rumble entries to compensate for
            // timer error.
            long elapsed = playbackWatch.ElapsedMilliseconds;
            long durationToWait = 0;
            while (true)
            {
                long rumbleEntryDuration = loadedRumble.durationsMs[rumbleIndex];
                long error = elapsed - rumblePositionMs;
                durationToWait = rumbleEntryDuration - error;

                // If durationToWait is <= 0, the current rumble entry needs to be skipped to
                // compensate for timer error. Otherwise break and play the current rumble entry.
                if (durationToWait > 0)
                {
                    break;
                }

                // If the end of the rumble has been reached, return, as playback has stopped.
                if (!IncreaseRumbleIndex())
                {
                    return;
                }
            }

            float lowFrequencySpeed = loadedRumble.lowFrequencyMotorSpeeds[rumbleIndex] * Mathf.Max(lowFrequencyMotorSpeedMultiplication, 0.0f);
            float highFrequencySpeed = loadedRumble.highFrequencyMotorSpeeds[rumbleIndex] * Mathf.Max(highFrequencyMotorSpeedMultiplication, 0.0f);

            UnityEngine.InputSystem.Gamepad currentGamepad = GetGamepad(currentGamepadID);
            // Check if gamepad was disconnected while playing
            if (currentGamepad != null)
            {
                currentGamepad.SetMotorSpeeds(lowFrequencySpeed, highFrequencySpeed);
            }
            else
            {
                return;
            }

            // Set up the timer to call ProcessNextRumble() again with the next rumble entry, after
            // the duration of the current rumble entry.
            rumblePositionMs += loadedRumble.durationsMs[rumbleIndex];
            rumbleIndex++;
            rumbleTimer.Interval = durationToWait;
            rumbleTimer.AutoReset = false;
            rumbleTimer.Enabled = true;
#endif
        }
    }
}
