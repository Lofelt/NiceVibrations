// Copyright (c) Meta Platforms, Inc. and affiliates. 

using UnityEngine;

namespace Lofelt.NiceVibrations
{
    /// <summary>
    /// Represents an imported haptic clip asset.
    /// </summary>
    ///
    /// HapticClip contains the data of a haptic clip asset imported from a <c>.haptic</c> file,
    /// in a format suitable for playing it back at runtime.
    /// A HapticClip is created by <c>HapticImporter</c> when importing a haptic clip asset
    /// in the Unity editor, and can be played back at runtime with e.g. HapticSource or
    /// HapticController::Play().
    ///
    /// It contains two representations:
    /// - JSON, used for playback on iOS and Android
    /// - GamepadRumble, used for playback on gamepads with the GamepadRumbler class
    public class HapticClip : ScriptableObject
    {
        /// <summary>
        /// The JSON representation of the haptic clip, stored as a byte array encoded in UTF-8,
        /// without a null terminator
        /// </summary>
        [SerializeField]
        public byte[] json;

        /// <summary>
        /// The haptic clip represented as a GamepadRumble struct
        /// </summary>
        [SerializeField]
        public GamepadRumble gamepadRumble;
    }
}
