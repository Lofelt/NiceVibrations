using UnityEngine;
using UnityEngine.UI;

namespace Lofelt.NiceVibrations
{
    public class EmphasisHapticsDemoManager : DemoManager
    {
        [Header("Emphasis Haptics")]
        public MMProgressBar AmplitudeProgressBar;
        public MMProgressBar FrequencyProgressBar;
        public HapticCurve TargetCurve;
        public float EmphasisAmplitude = 1f;
        public float EmphasisFrequency = 1f;
        public Text EmphasisAmplitudeText;
        public Text EmphasisFrequencyText;

        protected virtual void Start()
        {
            FrequencyProgressBar.UpdateBar(1f, 0f, 1f);
            AmplitudeProgressBar.UpdateBar(1f, 0f, 1f);
            TargetCurve.UpdateCurve(EmphasisAmplitude, EmphasisFrequency);

            HapticController.fallbackPreset = HapticPatterns.PresetType.RigidImpact;
        }

        public virtual void UpdateEmphasisAmplitude(float newAmplitude)
        {
            EmphasisAmplitude = newAmplitude;
            EmphasisAmplitudeText.text = NiceVibrationsDemoHelpers.Round(newAmplitude, 2).ToString();
            AmplitudeProgressBar.UpdateBar(EmphasisAmplitude, 0f, 1f);
            TargetCurve.UpdateCurve(EmphasisAmplitude, EmphasisFrequency);
        }

        public virtual void UpdateEmphasisFrequency(float newFrequency)
        {
            EmphasisFrequency = newFrequency;
            EmphasisFrequencyText.text = NiceVibrationsDemoHelpers.Round(newFrequency, 2).ToString();
            FrequencyProgressBar.UpdateBar(EmphasisFrequency, 0f, 1f);
            TargetCurve.UpdateCurve(EmphasisAmplitude, EmphasisFrequency);
        }

        public virtual void EmphasisHapticsButton()
        {
            HapticPatterns.PlayEmphasis(EmphasisAmplitude, EmphasisFrequency);
            StartCoroutine(Logo.Shake(0.2f));
            DebugAudioEmphasis.volume = EmphasisAmplitude;
            DebugAudioEmphasis.pitch = 0.5f + EmphasisFrequency / 2f;
            DebugAudioEmphasis.Play();
        }
    }
}
