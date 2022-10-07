// Copyright (c) Meta Platforms, Inc. and affiliates. 

using UnityEngine;

namespace Lofelt.NiceVibrations
{
    /// <summary>
    /// A <c>MonoBehaviour</c> that forwards global properties from HapticController and
    /// handles events
    /// </summary>
    ///
    /// While HapticSource provides a per-clip <c>MonoBehaviour</c> API for the functionality
    /// in HapticController, HapticReceiver provides a MonoBehaviour API for
    /// the global functionality in HapticController.
    ///
    /// HapticReceiver is also responsible for global event handling, such as an application
    /// focus change. To make this work correctly, your scene should have exactly one
    /// HapticReceiver component, similar to how a scene should have exactly one
    /// <c>AudioListener</c>.
    ///
    /// In the future HapticReceiver might receive parameters and distance to
    /// HapticSource components, and can be used for global parameter control through Unity
    /// Editor GUI.
    [AddComponentMenu("Nice Vibrations/Haptic Receiver")]
    public class HapticReceiver : MonoBehaviour, ISerializationCallbackReceiver
    {
        // These two fields are only used for serialization and deserialization.
        // HapticController manages the output haptic level and global haptic toggle,
        // HapticReceiver forwards these properties so they are available in a
        // MonoBehaviour.
        // To be able to serialize these properties, HapticReceiver needs to have
        // fields for them. Before serialization, these fields are set to the values
        // from HapticController, and after deserialization the values are restored
        // back to HapticController.
        [SerializeField]
        [Range(0.0f, 5.0f)]
        private float _outputLevel = 1.0f;
        [SerializeField]
        private bool _hapticsEnabled = true;

        /// <summary>
        /// Loads all fields from HapticController.
        /// </summary>
        public void OnBeforeSerialize()
        {
            _outputLevel = HapticController._outputLevel;
            _hapticsEnabled = HapticController._hapticsEnabled;
        }

        /// <summary>
        /// Writes all fields to HapticController.
        /// </summary>
        public void OnAfterDeserialize()
        {
            HapticController._outputLevel = _outputLevel;
            HapticController._hapticsEnabled = _hapticsEnabled;
        }

        /// <summary>
        /// Forwarded HapticController::outputLevel
        /// </summary>
        [System.ComponentModel.DefaultValue(1.0f)]
        public float outputLevel
        {
            get { return HapticController.outputLevel; }
            set { HapticController.outputLevel = value; }
        }


        /// <summary>
        /// Forwarded HapticController::hapticsEnabled
        /// </summary>
        [System.ComponentModel.DefaultValue(true)]
        public bool hapticsEnabled
        {
            get { return HapticController.hapticsEnabled; }
            set { HapticController.hapticsEnabled = value; }
        }

        /// <summary>
        /// Initializes HapticController.
        /// </summary>
        ///
        /// This ensures that the initialization time is spent at startup instead of when
        /// the first haptic is triggered during gameplay.
        void Start()
        {
            HapticController.Init();
        }

        /// <summary>
        /// Forwards an application focus change event to HapticController.
        /// </summary>
        void OnApplicationFocus(bool hasFocus)
        {
            HapticController.ProcessApplicationFocus(hasFocus);
        }

        /// <summary>
        /// Stops haptic playback on the gamepad when destroyed, to make sure the gamepad
        /// stops vibrating when quitting the application.
        /// </summary>
        void OnDestroy()
        {
            GamepadRumbler.Stop();
        }
    }
}
