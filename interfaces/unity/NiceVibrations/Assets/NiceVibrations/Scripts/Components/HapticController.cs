using UnityEngine;
using System;
using System.Timers;

#if (UNITY_ANDROID && !UNITY_EDITOR)
using System.Text;
#elif (UNITY_IOS && !UNITY_EDITOR)
using UnityEngine.iOS;
#endif

namespace Lofelt.NiceVibrations
{
    /// <summary>
    /// Provides haptic playback functionality.
    /// </summary>
    ///
    /// HapticController allows you to load and play <c>.haptic</c> clips, and
    /// provides various ways to control playback, such as seeking, looping and
    /// amplitude/frequency modulation.
    ///
    /// If you need a <c>MonoBehaviour</c> API, use HapticSource and
    /// HapticReceiver instead.
    ///
    /// On iOS and Android, the device is vibrated, using <c>LofeltHaptics</c>.
    /// On any platform, when a gamepad is connected, that gamepad is vibrated,
    /// using GamepadRumbler.
    ///
    /// Gamepads are vibrated automatically when HapticController detects that a
    /// gamepad is connected, no special code is needed to support gamepads.
    /// Gamepads only support Load(), Play(), Stop(), \ref clipLevel and \ref
    /// outputLevel. Other features like Seek(), Loop() and \ref clipFrequencyShift
    /// will have no effect on gamepads.
    ///
    /// None of the methods here are thread-safe and should only be called from
    /// the main (Unity) thread.
    public static class HapticController
    {
        static bool lofeltHapticsInitalized = false;

        // Timer used to call HandleFinishedPlayback() when playback is complete
        static Timer playbackFinishedTimer = new Timer();

        // Duration of the loaded haptic clip, in seconds
        static float clipLoadedDurationSecs = 0.0f;

        // Whether Load() has been called before
        static bool clipLoaded = false;

        // The value of the last call to seek()
        static float lastSeekTime = 0.0f;

        // Flag indicating if the device supports playing back .haptic clips
        static bool deviceMeetsAdvancedRequirements = false;

        // Flag indicating if the user enabled playback looping.
        // This does not necessarily mean that the currently active playback is looping, for
        // example gamepads don't support looping.
        static bool isLoopingEnabledByUser = false;

        // Flag indicating if the currently active playback is looping
        static bool isPlaybackLooping = false;

        static HapticPatterns.PresetType _fallbackPreset = HapticPatterns.PresetType.None;

        /// <summary>
        /// The haptic preset to be played when it's not possible to play a haptic clip
        /// </summary>
        public static HapticPatterns.PresetType fallbackPreset
        {
            get { return _fallbackPreset; }
            set { _fallbackPreset = value; }
        }

        internal static bool _hapticsEnabled = true;

        /// <summary>
        /// Property to enable and disable global haptic playback
        /// </summary>
        public static bool hapticsEnabled
        {
            get { return _hapticsEnabled; }
            set
            {
                if (_hapticsEnabled)
                {
                    Stop();
                }
                _hapticsEnabled = value;
            }
        }

        internal static float _outputLevel = 1.0f;

        /// <summary>
        /// The overall haptic output level
        /// </summary>
        ///
        /// It can be interpreted as the "volume control" for haptic playback.
        /// Output level is applied in combination with \ref clipLevel to the currently playing haptic clip.
        /// The combination of these two levels and the amplitude within the loaded haptic at a given moment
        /// in time determines the strength of the vibration felt on the device. \ref outputLevel is best used
        /// to increase or decrease the overall haptic level in a game.
        ///
        /// As output level pertains to all clips, unlike \ref clipLevel, it persists when a new clip is loaded.
        ///
        /// \ref outputLevel is a multiplication factor, it is <i>not</i> a dB value. The factor needs to be
        /// 0 or greater.
        ///
        /// The combination of \ref outputLevel and \ref clipLevel can result in a gain (for factors
        /// greater than 1.0) or an attenuation (for factors less than 1.0) to the clip. If the
        /// combination of \ref outputLevel, \ref clipLevel and the amplitude within the loaded haptic
        /// is greater than 1.0, it is clipped to 1.0. Hard clipping is performed, no limiter is used.
        ///
        /// On Android, an adjustment to \ref outputLevel will take effect in the next call to Play().
        /// On iOS, it will take effect right away.
        [System.ComponentModel.DefaultValue(1.0f)]
        public static float outputLevel
        {
            get { return _outputLevel; }
            set
            {
                _outputLevel = value;

                if (Init())
                {
                    LofeltHaptics.SetAmplitudeMultiplication(_outputLevel * _clipLevel);
                }
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
                GamepadRumbler.lowFrequencyMotorSpeedMultiplication = _outputLevel * _clipLevel;
                GamepadRumbler.highFrequencyMotorSpeedMultiplication = _outputLevel * _clipLevel;
#endif
            }
        }

        internal static float _clipLevel = 1.0f;

        /// <summary>
        /// The level of the loaded clip
        /// </summary>
        ///
        /// Clip level is applied in combination with \ref outputLevel, to the
        /// currently playing haptic clip. The combination of these two levels and the amplitude within the loaded
        /// haptic at a given moment in time determines the strength of the vibration felt on the device.
        /// \ref clipLevel is best used to adjust the level of a single clip based on game state.
        ///
        /// As clip level is specific to an individual clip, unlike \ref outputLevel, it resets to
        /// 1.0 when a new clip is loaded.
        ///
        /// \ref clipLevel is a multiplication factor, it is <i>not</i> a dB value. The factor needs to be
        /// 0 or greater.
        ///
        /// The combination of \ref outputLevel and \ref clipLevel can result in a gain (for factors
        /// greater than 1.0) or an attenuation (for factors less than 1.0) to the clip.
        ///
        /// If the combination of \ref outputLevel, \ref clipLevel and the amplitude within the loaded
        /// haptic is greater than 1.0, it is clipped to 1.0. Hard clipping is performed, no limiter is used.
        ///
        /// The clip needs to be loaded with Load() before adjusting \ref clipLevel. Loading a clip
        /// resets \ref clipLevel back to the default of 1.0.
        ///
        /// On Android, an adjustment to \ref clipLevel will take effect in the next call to Play(). On iOS,
        /// it will take effect right away.
        ///
        /// On Android, setting the clip level should be done before calling \ref Seek(), since
        /// setting a clip level ignores the sought value.
        ///
        [System.ComponentModel.DefaultValue(1.0f)]
        public static float clipLevel
        {
            get { return _clipLevel; }
            set
            {
                _clipLevel = value;

                if (Init())
                {
                    LofeltHaptics.SetAmplitudeMultiplication(_outputLevel * _clipLevel);
                }
#if ((!UNITY_ANDROID && !UNITY_IOS) || UNITY_EDITOR) && NICE_VIBRATIONS_INPUTSYSTEM_INSTALLED && ENABLE_INPUT_SYSTEM && !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
                GamepadRumbler.lowFrequencyMotorSpeedMultiplication = _outputLevel * _clipLevel;
                GamepadRumbler.highFrequencyMotorSpeedMultiplication = _outputLevel * _clipLevel;
#endif
            }
        }

        /// Action that is invoked when Load() is called
        public static Action LoadedClipChanged;

        /// Action that is invoked when Play() is called
        public static Action PlaybackStarted;

        /// <summary>
        /// Action that is invoked when the playback has finished
        /// </summary>
        ///
        /// This happens either when Stop() is explicitly called, or when a non-looping
        /// clip has finished playing.
        ///
        /// This can be invoked spuriously, even if no haptics are currently playing, for example
        /// if Stop() is called multiple times in a row.
        public static Action PlaybackStopped;

        /// <summary>
        /// Initializes HapticController.
        /// </summary>
        ///
        /// Calling this method multiple times has no effect and is safe.
        ///
        /// You do not need to call this method, HapticController automatically calls this
        /// method before any operation that needs initialization, such as Play().
        /// However it can be beneficial to call this early during startup, so the initialization
        /// time is spent at startup instead of when the first haptic is triggered during gameplay.
        /// If you have a HapticReceiver in your scene, it takes care of calling
        /// Init() during startup for you.
        ///
        /// Do not call this method from a static constructor. Unity often invokes static
        /// constructors from a different thread, for example during deserialization. The
        /// initialization code is not thread-safe. This is the reason this method is not called
        /// from the static constructor of HapticController or HapticReceiver.
        ///
        /// <returns>Whether the device supports the minimum requirements to play haptics</returns>
        public static bool Init()
        {
            if (!lofeltHapticsInitalized)
            {
                lofeltHapticsInitalized = true;

                var syncContext = System.Threading.SynchronizationContext.Current;
                playbackFinishedTimer.Elapsed += (object obj, System.Timers.ElapsedEventArgs args) =>
                {
                    // Timer elapsed events are called from a separate thread, so use
                    // SynchronizationContext to handle it in the main thread.
                    syncContext.Post(_ =>
                    {
                        HandleFinishedPlayback();
                    }, null);
                };

                if (DeviceCapabilities.isVersionSupported)
                {
                    LofeltHaptics.Initialize();
                    DeviceCapabilities.Init();
                    deviceMeetsAdvancedRequirements = DeviceCapabilities.meetsAdvancedRequirements;
                }

                GamepadRumbler.Init();
            }
            return deviceMeetsAdvancedRequirements;
        }

        /// <summary>
        /// Loads a haptic clip given in JSON format for later playback.
        /// </summary>
        ///
        /// This overload of Load() is useful in cases there is only the JSON data of a haptic clip
        /// available. Due to only having the JSON data and no GamepadRumble, gamepad playback is
        /// not supported with this overload.
        ///
        /// <param name="data">The haptic clip, which is the content of the
        /// <c>.haptic</c> file, a UTF-8 encoded JSON string without a null
        /// terminator</param>
        public static void Load(byte[] data)
        {
            GamepadRumbler.Unload();
            lastSeekTime = 0.0f;
            clipLoaded = true;
            clipLoadedDurationSecs = 0.0f;
            if (Init())
            {
                LofeltHaptics.Load(data);
            }
            clipLevel = 1.0f;
            LoadedClipChanged?.Invoke();
        }

        /// <summary>
        /// Loads the given HapticClip for later playback.
        /// </summary>
        ///
        /// This is the standard way to load a haptic clip, while the other overloads of Load()
        /// are for more specialized cases.
        ///
        /// At the moment only one clip can be loaded at a time.
        ///
        /// <param name="clip">The HapticClip to be loaded</param>
        public static void Load(HapticClip clip)
        {
            Load(clip.json, clip.gamepadRumble);
        }

        /// <summary>
        /// Loads the haptic clip given as JSON and GamepadRumble for later playback.
        /// </summary>
        ///
        /// This is an overload of Load() that is useful when a HapticClip is not available, and
        /// both the JSON and GamepadRumble are. One such case is generating both dynamically at
        /// runtime.
        ///
        /// <param name="json">The haptic clip, which is the content of the <c>.haptic</c> file,
        /// a UTF-8 encoded JSON string without a null terminator</param>
        /// <param name="rumble">The GamepadRumble representation of the haptic clip</param>
        public static void Load(byte[] json, GamepadRumble rumble)
        {
            Load(json);
            GamepadRumbler.Load(rumble);

            // Load() only sets the correct clip duration on iOS and Android, and sets it to 0.0
            // on other platforms. For the other platforms, set a clip duration based on the
            // GamepadRumble here.
            if (clipLoadedDurationSecs == 0.0f && rumble.IsValid())
            {
                clipLoadedDurationSecs = rumble.totalDurationMs / 1000.0f;
            }
        }

        static void HandleFinishedPlayback()
        {
            lastSeekTime = 0.0f;
            isPlaybackLooping = false;
            playbackFinishedTimer.Enabled = false;
            PlaybackStopped?.Invoke();
        }

        /// <summary>
        /// Plays the haptic clip that was previously loaded with Load().
        /// </summary>
        ///
        /// If <c>Loop(true)</c> was called previously, the playback will be repeated
        /// until Stop() is called. Otherwise the haptic clip will only play once.
        ///
        /// In case the device does not meet the requirements to play <c>.haptic</c> clips, this
        /// function will call HapticPatterns.PlayPreset() with the \ref fallbackPreset set. In this
        /// case, functionality like seeking, looping and runtime modulation won't do anything as
        /// they aren't available for haptic presets.
        public static void Play()
        {
            if (!_hapticsEnabled)
            {
                return;
            }

            float remainingPlayDuration = 0.0f;
            bool canLoop = false;
            if (GamepadRumbler.CanPlay())
            {
                remainingPlayDuration = clipLoadedDurationSecs;
                GamepadRumbler.Play();
            }
            else if (Init())
            {
                remainingPlayDuration = Mathf.Max(clipLoadedDurationSecs - lastSeekTime, 0.0f);
                canLoop = DeviceCapabilities.canLoop;
                LofeltHaptics.Play();
            }
            else if (DeviceCapabilities.isVersionSupported)
            {
                remainingPlayDuration = HapticPatterns.GetPresetDuration(fallbackPreset);
                HapticPatterns.PlayPreset(fallbackPreset);
            }

            isPlaybackLooping = isLoopingEnabledByUser && canLoop;
            PlaybackStarted?.Invoke();

            //
            // Call HandleFinishedPlayback() after the playback finishes
            //
            if (remainingPlayDuration > 0.0f)
            {
                playbackFinishedTimer.Interval = remainingPlayDuration * 1000;
                playbackFinishedTimer.AutoReset = false;
                playbackFinishedTimer.Enabled = !isPlaybackLooping;
            }
            else
            {
                // Setting playbackFinishedTimer.Interval needs an interval > 0, otherwise it will
                // throw an exception.
                // Even if the remaining play duration is 0, we still want to trigger everything
                // that happens in HandleFinishedPlayback().
                // A playback duration of 0 happens in the Unity editor, when loading the clip
                // failed or when seeking to the end of a clip.
                HandleFinishedPlayback();
            }
        }


        /// <summary>
        /// Loads and plays the HapticClip given as an argument.
        /// </summary>
        ///
        /// <param name="clip">The HapticClip to be played</param>
        public static void Play(HapticClip clip)
        {
            Load(clip);
            Play();
        }

        /// <summary>
        /// Stops haptic playback
        ///
        /// </summary>
        public static void Stop()
        {

            if (Init())
            {
                LofeltHaptics.Stop();
            }
            else
            {
                LofeltHaptics.StopPattern();
            }
            GamepadRumbler.Stop();
            HandleFinishedPlayback();
        }

        /// <summary>
        /// Jumps to a time position in the haptic clip.
        /// </summary>
        ///
        /// The playback will always be stopped when this function is called.
        /// This is to match the behavior between iOS and Android, since Android needs to
        /// restart playback for seek to have effect.
        ///
        /// If seeking beyond the end of the clip, Play() will not reproduce any haptics.
        /// Seeking to a negative position will seek to the beginning of the clip.
        ///
        /// <param name="time">The new position within the clip, as seconds from the beginning
        /// of the clip</param>
        public static void Seek(float time)
        {
            if (Init())
            {
                LofeltHaptics.Stop();
                LofeltHaptics.Seek(time);
            }
            GamepadRumbler.Stop();
            lastSeekTime = time;
        }

        /// <summary>
        /// Adds the given shift to the frequency of every breakpoint in the clip, including the
        /// emphasis.
        /// </summary>
        ///
        /// In other words, this property shifts all frequencies of the clip. The frequency shift is
        /// added to each frequency value and needs to be between -1.0 and 1.0. If the resulting
        /// frequency of a breakpoint is smaller than 0.0 or greater than 1.0, it is clipped to that
        /// range. The frequency is clipped hard, no limiter is used.
        ///
        /// The clip needs to be loaded with Load() first. Loading a clip resets the shift back
        /// to the default of 0.0.
        ///
        /// Setting the frequency shift has no effect on Android; it only works on iOS.
        ///
        /// A call to this property will change the frequency shift of a currently playing clip
        /// right away. If no clip is playing, the shift is applied in the next call to
        /// Play().
        [System.ComponentModel.DefaultValue(0.0f)]
        public static float clipFrequencyShift
        {
            set
            {
                if (Init())
                {
                    LofeltHaptics.SetFrequencyShift(value);
                }
            }
        }

        /// <summary>
        /// Set the playback of a haptic clip to loop.
        /// </summary>
        ///
        /// On Android, calling this will always put the playback position at the start of the clip.
        /// Also, it will only have an effect when Play() is called again.
        ///
        /// On iOS, if a clip is already playing, calling this will leave the playback position as
        /// it is and repeat when it reaches the end. No need to call Play() again for
        /// changes to take effect.
        ///
        /// <param name="enabled">If the value is <c>true</c>, looping will be enabled which results
        /// in repeating the playback until Stop() is called; if <c>false</c>, the haptic
        /// clip will only be played once.</param>
        public static void Loop(bool enabled)
        {
            if (Init())
            {
                LofeltHaptics.Loop(enabled);
            }
            isLoopingEnabledByUser = enabled;
        }

        /// <summary>
        /// Checks if the loaded haptic clip is playing.
        /// </summary>
        ///
        /// <returns>Whether the loaded clip is playing</returns>
        public static bool IsPlaying()
        {
            if (playbackFinishedTimer.Enabled)
            {
                return true;
            }
            else
            {
                return isPlaybackLooping;
            }
        }

        /// <summary>
        /// Stops playback and resets the playback state.
        /// </summary>
        ///
        /// Seek position, clip level, clip frequency shift and loop are reset to the
        /// default values.
        /// The currently loaded clip stays loaded.
        /// \ref hapticsEnabled and \ref outputLevel are not reset.
        public static void Reset()
        {
            if (clipLoaded)
            {
                Seek(0.0f);
                Stop();
                clipLevel = 1.0f;
                clipFrequencyShift = 0.0f;
                Loop(false);
            }
            fallbackPreset = HapticPatterns.PresetType.None;
        }

        /// <summary>
        /// Processes an application focus change event.
        /// </summary>
        ///
        /// If you have a HapticReceiver in your scene, the HapticReceiver
        /// will take care of calling this method when needed. Otherwise it is your
        /// responsibility to do so.
        ///
        /// When the application loses the focus, playback is stopped.
        ///
        /// <param name="hasFocus">Whether the application now has focus</param>
        public static void ProcessApplicationFocus(bool hasFocus)
        {
            if (!hasFocus)
            {
                // While LofeltHaptics stops playback when the app loses focus,
                // calling Stop() here handles additional things such as invoking
                // the PlaybackStopped Action.
                Stop();
            }
        }
    }
}
