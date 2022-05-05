using UnityEngine;

namespace Lofelt.NiceVibrations
{
    /// <summary>
    /// Provides haptic playback functionality for a single haptic clip.
    /// </summary>
    ///
    /// HapticSource plays back the HapticClip assigned in the \ref clip property
    /// when calling Play(). It also provides various ways to control playback, such as
    /// seeking, looping and amplitude/frequency modulation.
    ///
    /// When a gamepad is connected, the haptic clip will be played back on that gamepad.
    /// See the HapticController documentation for more details about gamepad support.
    ///
    /// At the moment, playback of a haptic source is not triggered automatically
    /// by e.g. proximity between the HapticReceiver and the HapticSource,
    /// so you need to call Play() to trigger playback.
    ///
    /// You can place multiple HapticSource components in your scene, with a different
    /// HapticClip assigned to each.
    ///
    /// HapticSource provides a per-clip <c>MonoBehaviour</c> API for the functionality
    /// in HapticController, while HapticReceiver provides a <c>MonoBehaviour</c> API
    /// for the global functionality in HapticController.
    ///
    /// <c>HapticSourceInspector</c> provides a custom editor for HapticSource for the
    /// Inspector.
    [AddComponentMenu("Nice Vibrations/Haptic Source")]
    public class HapticSource : MonoBehaviour
    {
        const int DEFAULT_PRIORITY = 128;

        /// The HapticClip this HapticSource loads and plays.
        public HapticClip clip;

        /// <summary>
        /// The priority of the HapticSource
        /// </summary>
        ///
        /// This property is set by <c>HapticSourceInspector</c>. 0 is the highest priority and 256
        /// is the lowest priority.
        ///
        /// The default value is 128.
        public int priority = DEFAULT_PRIORITY;

        /// <summary>
        /// Jump in time position of haptic source playback.
        /// </summary>
        ///
        /// Initially set to 0.0 seconds.
        /// This value can only be set when using Seek().
        float seekTime = 0.0f;

        [SerializeField]
        HapticPatterns.PresetType _fallbackPreset = HapticPatterns.PresetType.None;

        /// <summary>
        /// The haptic preset to be played when it's not possible to play a haptic clip
        /// </summary>
        [System.ComponentModel.DefaultValue(HapticPatterns.PresetType.None)]
        public HapticPatterns.PresetType fallbackPreset
        {
            get { return _fallbackPreset; }
            set { _fallbackPreset = value; }
        }

        [SerializeField]
        bool _loop = false;

        /// <summary>
        /// Set the haptic source to loop playback of the haptic clip.
        /// </summary>
        ///
        /// It will only have any effect once Play() is called.
        ///
        /// See HapticController::Loop() for further details.
        [System.ComponentModel.DefaultValue(false)]
        public bool loop
        {
            get { return _loop; }
            set { _loop = value; }
        }

        [SerializeField]
        float _level = 1.0f;

        /// <summary>
        /// The level of the haptic source
        /// </summary>
        ///
        /// Haptic source level is applied in combination with output level (which can be set on either
        /// HapticReceiver or HapticController according to preference), to the currently playing
        /// haptic clip. The combination of these two levels and the amplitude within the loaded
        /// haptic at a given moment in time determines the strength of the vibration felt on the device. See
        /// HapticController::clipLevel for further details.
        [System.ComponentModel.DefaultValue(1.0)]
        public float level
        {
            get { return _level; }
            set
            {
                _level = value;

                if (IsLoaded())
                {
                    HapticController.clipLevel = _level;
                }
            }
        }

        [SerializeField]
        float _frequencyShift = 0.0f;

        /// <summary>
        /// This shift is added to the frequency of every breakpoint in the clip, including the
        /// emphasis.
        /// </summary>
        ///
        /// See HapticController::clipFrequencyShift for further details.
        [System.ComponentModel.DefaultValue(0.0)]
        public float frequencyShift
        {
            get { return _frequencyShift; }
            set
            {
                _frequencyShift = value;

                if (IsLoaded())
                {
                    HapticController.clipFrequencyShift = _frequencyShift;
                }
            }
        }

        /// The HapticSource that is currently loaded into HapticController.
        /// This can be null if nothing was ever loaded, or if HapticController::Load()
        /// was called directly, bypassing HapticSource.
        static HapticSource loadedHapticSource = null;

        /// The HapticSource that was last played.
        /// This can be null if nothing was ever player, or if HapticController::Play()
        /// was called directly, bypassing HapticSource.
        /// The lastPlayedHapticSource isn't necessarily playing now, lastPlayedHapticSource
        /// will remain set even if playback has finished or was stopped.
        static HapticSource lastPlayedHapticSource = null;

        static HapticSource()
        {
            // When HapticController::Load() or HapticController::Play() is
            // called directly, bypassing HapticSource, reset loadedHapticSource
            // and lastPlayedHapticSource.
            HapticController.LoadedClipChanged += () =>
            {
                loadedHapticSource = null;
            };
            HapticController.PlaybackStarted += () =>
            {
                lastPlayedHapticSource = null;
            };
        }

        /// <summary>
        /// Loads and plays back the haptic clip.
        /// </summary>
        ///
        /// At the moment only one haptic clip at a time can be played. If another
        /// HapticSource is currently playing and has lower priority, its playback will
        /// be stopped.
        ///
        /// If a seek time within the time range of the clip has been set with Seek(),
        /// it will jump to that position if \ref loop is <c>false</c>. If \ref loop
        /// is <c>true</c>, seeking will have no effect.
        ///
        /// It will loop playback in case \ref loop is <c>true</c>.
        public void Play()
        {
            if (CanPlay())
            {
                //
                // Load
                //
                HapticController.Load(clip);
                loadedHapticSource = this;

                //
                // Apply properties like loop, modulation and seek position
                //
                HapticController.Loop(loop);

                HapticController.clipLevel = level;
                HapticController.clipFrequencyShift = frequencyShift;

                if (seekTime != 0.0f && !loop)
                {
                    HapticController.Seek(seekTime);
                }

                //
                // Play
                //
                HapticController.fallbackPreset = fallbackPreset;
                HapticController.Play();
                lastPlayedHapticSource = this;
            }
        }

        private bool CanPlay()
        {
            return (!HapticController.IsPlaying() ||
                   (lastPlayedHapticSource != null && priority <= lastPlayedHapticSource.priority));
        }

        /// <summary>
        /// Checks if the current HapticSource has been loaded into HapticController.
        /// </summary>
        ///
        /// This is used to avoid triggering operations on HapticController while
        /// another HapticSource is loaded.
        private bool IsLoaded()
        {
            return Object.ReferenceEquals(this, loadedHapticSource);
        }

        /// <summary>
        /// Stops playback that was previously started with Play().
        /// </summary>
        public void Stop()
        {
            if (IsLoaded())
            {
                HapticController.Stop();
            }
        }

        /// <summary>
        /// Sets the time position to jump to when Play() is called.
        /// </summary>
        ///
        /// It will only have an effect once Play() is called.
        ///
        /// <param name="time">The position in the clip, in seconds</param>
        public void Seek(float time)
        {
            this.seekTime = time;
        }

        /// <summary>
        /// When a <c>GameObject</c> is disabled, stop playback if this HapticSource is
        /// playing.
        /// </summary>
        public void OnDisable()
        {
            if (HapticController.IsPlaying() && IsLoaded())
            {
                this.Stop();
            }
        }
    }
}
