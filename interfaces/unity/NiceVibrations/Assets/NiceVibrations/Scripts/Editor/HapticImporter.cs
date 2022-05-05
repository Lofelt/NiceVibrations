using System.IO;
using System.Runtime.InteropServices;
using System;
using UnityEngine;
using System.Text;

#if UNITY_2020_2_OR_NEWER
using UnityEditor.AssetImporters;
#elif UNITY_2019_4_OR_NEWER
using UnityEditor.Experimental.AssetImporters;
#endif

namespace Lofelt.NiceVibrations
{
    [ScriptedImporter(version: 3, ext: "haptic", AllowCaching = true)]
    /// <summary>
    /// Provides an importer for the HapticClip component.
    /// </summary>
    ///
    /// The importer takes a <c>.haptic</c> file and converts it into a HapticClip.
    public class HapticImporter : ScriptedImporter
    {
#if !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
        [DllImport("nice_vibrations_editor_plugin")]
        private static extern IntPtr nv_plugin_convert_haptic_to_gamepad_rumble([In] byte[] bytes, long size);

        [DllImport("nice_vibrations_editor_plugin")]
        private static extern void nv_plugin_destroy(IntPtr gamepadRumble);

        [DllImport("nice_vibrations_editor_plugin")]
        private static extern UIntPtr nv_plugin_get_length(IntPtr gamepadRumble);

        [DllImport("nice_vibrations_editor_plugin")]
        private static extern void nv_plugin_get_durations(IntPtr gamepadRumble, [Out] int[] durations);

        [DllImport("nice_vibrations_editor_plugin")]
        private static extern void nv_plugin_get_low_frequency_motor_speeds(IntPtr gamepadRumble, [Out] float[] lowFrequencies);

        [DllImport("nice_vibrations_editor_plugin")]
        private static extern void nv_plugin_get_high_frequency_motor_speeds(IntPtr gamepadRumble, [Out] float[] highFrequencies);

        // We can not use "[return: MarshalAs(UnmanagedType.LPUTF8Str)]" here, and have to use
        // IntPtr for the return type instead. Otherwise the C# runtime tries to free the returned
        // string, which is invalid as the native plugin keeps ownership of the string.
        // We use PtrToStringUTF8() to manually convert the IntPtr to a string instead.
        [DllImport("nice_vibrations_editor_plugin")]
        private static extern IntPtr nv_plugin_get_last_error();

        [DllImport("nice_vibrations_editor_plugin")]
        private static extern UIntPtr nv_plugin_get_last_error_length();

        // Alternative to Marshal.PtrToStringUTF8() which was introduced in .NET 5 and isn't yet
        // supported by Unity
        private string PtrToStringUTF8(IntPtr ptr, int length)
        {
            byte[] bytes = new byte[length];
            Marshal.Copy(ptr, bytes, 0, length);
            return Encoding.UTF8.GetString(bytes, 0, length);
        }
#endif

        public override void OnImportAsset(AssetImportContext ctx)
        {
            // Load .haptic clip from file
            var fileName = System.IO.Path.GetFileNameWithoutExtension(ctx.assetPath);
            var jsonBytes = File.ReadAllBytes(ctx.assetPath);
            var hapticClip = HapticClip.CreateInstance<HapticClip>();
            hapticClip.json = jsonBytes;

#if !NICE_VIBRATIONS_DISABLE_GAMEPAD_SUPPORT
            // Convert JSON to a GamepadRumble struct. The conversion algorithm is inside the native
            // library nice_vibrations_editor_plugin. That plugin is only used in the Unity editor, and
            // not at runtime.
            GamepadRumble rumble = default;
            IntPtr nativeRumble = nv_plugin_convert_haptic_to_gamepad_rumble(jsonBytes, jsonBytes.Length);
            if (nativeRumble != IntPtr.Zero)
            {
                try
                {
                    uint length = (uint)nv_plugin_get_length(nativeRumble);
                    rumble.durationsMs = new int[length];
                    rumble.lowFrequencyMotorSpeeds = new float[length];
                    rumble.highFrequencyMotorSpeeds = new float[length];

                    nv_plugin_get_durations(nativeRumble, rumble.durationsMs);
                    nv_plugin_get_low_frequency_motor_speeds(nativeRumble, rumble.lowFrequencyMotorSpeeds);
                    nv_plugin_get_high_frequency_motor_speeds(nativeRumble, rumble.highFrequencyMotorSpeeds);

                    int totalDurationMs = 0;
                    foreach (int duration in rumble.durationsMs)
                    {
                        totalDurationMs += duration;
                    }
                    rumble.totalDurationMs = totalDurationMs;
                }
                finally
                {
                    nv_plugin_destroy(nativeRumble);
                }
            }
            else
            {
                var lastErrorPtr = nv_plugin_get_last_error();
                var lastErrorLength = (int)nv_plugin_get_last_error_length();
                var lastError = PtrToStringUTF8(lastErrorPtr, lastErrorLength);
                Debug.LogWarning($"Failed to convert haptic clip {ctx.assetPath} to gamepad rumble: {lastError}");
            }

            hapticClip.gamepadRumble = rumble;
#endif

            // Use hapticClip as the imported asset
            ctx.AddObjectToAsset("com.lofelt.HapticClip", hapticClip);
            ctx.SetMainObject(hapticClip);
        }
    }
}
