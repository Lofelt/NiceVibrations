// Copyright (c) Meta Platforms, Inc. and affiliates. 

using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;

namespace Lofelt.NiceVibrations
{
    public class ContinuousHapticsDemoManager : DemoManager
    {
        [Header("Texts")]
        public float ContinuousAmplitude = 1f;
        public float ContinuousFrequency = 1f;
        public float ContinuousDuration = 3f;
        public Text ContinuousAmplitudeText;
        public Text ContinuousFrequencyText;
        public Text ContinuousDurationText;
        public Text ContinuousButtonText;
        [Header("Interface")]
        public MMTouchButton ContinuousButton;
        public MMProgressBar AmplitudeProgressBar;
        public MMProgressBar FrequencyProgressBar;
        public MMProgressBar DurationProgressBar;
        public MMProgressBar ContinuousProgressBar;
        public HapticCurve TargetCurve;
        public Slider DurationSlider;

        protected float _timeLeft;
        protected Color _continuousButtonOnColor = new Color32(216, 85, 85, 255);
        protected Color _continuousButtonOffColor = new Color32(242, 27, 80, 255);
        protected bool _continuousActive = false;
        protected float _amplitudeLastFrame = -1f;
        protected float _frequencyLastFrame = -1f;

        protected virtual void Awake()
        {
            ContinuousButton.ReturnToInitialSpriteAutomatically = false;

            ContinuousAmplitudeText.text = ContinuousAmplitude.ToString();
            ContinuousFrequencyText.text = ContinuousFrequency.ToString();
            ContinuousDurationText.text = ContinuousDuration.ToString();

            AmplitudeProgressBar.UpdateBar(ContinuousAmplitude, 0f, 1f);
            FrequencyProgressBar.UpdateBar(ContinuousFrequency, 0f, 1f);
            DurationProgressBar.UpdateBar(ContinuousDuration, 0f, 5f);
        }

        protected virtual void Update()
        {
            UpdateContinuousDemo();
        }

        protected virtual void UpdateContinuousDemo()
        {
            if (_timeLeft > 0f)
            {
                ContinuousProgressBar.UpdateBar(_timeLeft, 0f, ContinuousDuration);
                _timeLeft -= Time.deltaTime;
                Logo.Shaking = true;
                TargetCurve.Move = true;
                Logo.Amplitude = NiceVibrationsDemoHelpers.Remap(ContinuousAmplitude, 0f, 1f, 1f, 8f);
                Logo.Frequency = NiceVibrationsDemoHelpers.Remap(ContinuousFrequency, 0f, 1f, 10f, 25f);
            }
            else
            {
                ContinuousProgressBar.UpdateBar(0f, 0f, ContinuousDuration);
                Logo.Shaking = false;
                TargetCurve.Move = false;
                if (_continuousActive)
                {
                    HapticController.Stop();
                }
            }
            if ((_frequencyLastFrame != ContinuousFrequency) || (_amplitudeLastFrame != ContinuousAmplitude))
            {
                TargetCurve.UpdateCurve(ContinuousAmplitude, ContinuousFrequency);
            }
            _amplitudeLastFrame = ContinuousAmplitude;
            _frequencyLastFrame = ContinuousFrequency;
        }

        public virtual void UpdateContinuousAmplitude(float newAmplitude)
        {
            ContinuousAmplitude = newAmplitude;
            AmplitudeProgressBar.UpdateBar(ContinuousAmplitude, 0f, 1f);
            ContinuousAmplitudeText.text = NiceVibrationsDemoHelpers.Round(newAmplitude, 2).ToString();
            UpdateContinuous();
        }

        public virtual void UpdateContinuousFrequency(float newFrequency)
        {
            ContinuousFrequency = newFrequency;
            FrequencyProgressBar.UpdateBar(ContinuousFrequency, 0f, 1f);
            ContinuousFrequencyText.text = NiceVibrationsDemoHelpers.Round(newFrequency, 2).ToString();
            UpdateContinuous();
        }

        public virtual void UpdateContinuousDuration(float newDuration)
        {
            ContinuousDuration = newDuration;
            DurationProgressBar.UpdateBar(ContinuousDuration, 0f, 5f);
            ContinuousDurationText.text = NiceVibrationsDemoHelpers.Round(newDuration, 2).ToString();
        }

        protected virtual void UpdateContinuous()
        {
            if (_continuousActive)
            {
                HapticController.clipLevel = ContinuousAmplitude;
                HapticController.clipFrequencyShift = ContinuousFrequency;
                DebugAudioContinuous.volume = ContinuousAmplitude;
                DebugAudioContinuous.pitch = 0.5f + ContinuousFrequency / 2f;
            }
        }

        public virtual void ContinuousHapticsButton()
        {
            if (!_continuousActive)
            {
                // START
                HapticController.fallbackPreset = HapticPatterns.PresetType.LightImpact;
                HapticPatterns.PlayConstant(ContinuousAmplitude, ContinuousFrequency, ContinuousDuration);
                _timeLeft = ContinuousDuration;
                ContinuousButtonText.text = "Stop continuous haptic pattern";
                DurationSlider.interactable = false;
                _continuousActive = true;
                DebugAudioContinuous.Play();
            }
            else
            {
                // STOP
                HapticController.Stop();
                ResetPlayState();
            }
        }

        protected virtual void OnHapticsStopped()
        {
            ResetPlayState();
        }

        protected virtual void ResetPlayState()
        {
            _timeLeft = 0f;
            ContinuousButtonText.text = "Play continuous haptic pattern";
            _continuousActive = false;
            DebugAudioContinuous?.Stop();
            DurationSlider.interactable = true;

        }

        protected virtual void OnEnable()
        {
            HapticController.PlaybackStopped += OnHapticsStopped;
        }

        protected virtual void OnDisable()
        {
            HapticController.PlaybackStopped -= OnHapticsStopped;
        }
    }
}
