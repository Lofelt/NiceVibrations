using System.Collections.Generic;
using UnityEngine;

namespace Lofelt.NiceVibrations
{
    public class CarDemoManager : DemoManager
    {
        [Header("Control")]
        public MMKnob Knob;
        public float MinimumKnobValue = 0.1f;
        public float MaximumPowerDuration = 10f;
        public float ChargingSpeed = 2f;
        public float CarSpeed = 0f;
        public float Power;
        public float StartClickDuration = 0.2f;
        public float DentDuration = 0.10f;
        public List<float> Dents;

        [Header("Car")]

        public AudioSource CarEngineAudioSource;
        public Transform LeftWheel;
        public Transform RightWheel;
        public RectTransform CarBody;
        public Vector3 WheelRotationSpeed = new Vector3(0f, 0f, 50f);

        [Header("UI")]
        public GameObject ReloadingPrompt;
        public AnimationCurve StartClickCurve;
        public MMProgressBar PowerBar;
        public List<PowerBarElement> SpeedBars;
        public Color ActiveColor;
        public Color InactiveColor;

        [Header("Debug")]
        public bool _carStarted = false;
        public float _carStartedAt = 0f;
        public float _lastStartClickAt = 0f;

        protected float _knobValueLastFrame;
        protected float _lastDentAt = 0f;
        protected float _knobValue;
        protected Vector3 _initialCarPosition;
        protected Vector3 _carPosition;

        protected virtual void Awake()
        {
            Power = MaximumPowerDuration;
            ReloadingPrompt.SetActive(false);
            _initialCarPosition = CarBody.localPosition;
        }

        protected virtual void Update()
        {
            HandlePower();
            UpdateCar();
            UpdateUI();

            _knobValueLastFrame = Knob.Value;
        }

        protected virtual void HandlePower()
        {
            _knobValue = Knob.Active ? Knob.Value : 0f;

            if (!_carStarted)
            {
                if ((_knobValue > MinimumKnobValue) && (Knob.Active))
                {
                    _carStarted = true;
                    _carStartedAt = Time.time;
                    _lastStartClickAt = Time.time;

                    HapticPatterns.PlayConstant(_knobValue, _knobValue, MaximumPowerDuration);
                    CarEngineAudioSource.Play();
                }
                else
                {
                    Power += Time.deltaTime * ChargingSpeed;
                    Power = Mathf.Clamp(Power, 0f, MaximumPowerDuration);

                    if (Power == MaximumPowerDuration)
                    {
                        Knob.SetActive(true);
                        Knob._rectTransform.localScale = Vector3.one;
                        ReloadingPrompt.SetActive(false);
                    }
                    else
                    {
                        if (!Knob.Active)
                        {
                            Knob.SetValue(CarSpeed);
                        }
                    }
                }
            }
            else
            {
                if (Time.time - _carStartedAt > MaximumPowerDuration)
                {
                    _carStarted = false;
                    Knob.SetActive(false);
                    Knob._rectTransform.localScale = Vector3.one * 0.9f;
                    ReloadingPrompt.SetActive(true);
                }
                else
                {
                    if (_knobValue > MinimumKnobValue)
                    {
                        Power -= Time.deltaTime;
                        Power = Mathf.Clamp(Power, 0f, MaximumPowerDuration);

                        HapticController.clipLevel = _knobValue;
                        HapticController.clipFrequencyShift = _knobValue;

                        if (Power <= 0f)
                        {
                            _carStarted = false;
                            Knob.SetActive(false);
                            Knob._rectTransform.localScale = Vector3.one * 0.9f;
                            ReloadingPrompt.SetActive(true);
                            HapticController.Stop();
                        }
                    }
                    else
                    {
                        _carStarted = false;
                        _lastStartClickAt = Time.time;
                        HapticController.Stop();
                    }
                }
            }
        }

        protected virtual void UpdateCar()
        {
            float targetSpeed = _carStarted ? NiceVibrationsDemoHelpers.Remap(Knob.Value, MinimumKnobValue, 1f, 0f, 1f) : 0f;
            CarSpeed = Mathf.Lerp(CarSpeed, targetSpeed, Time.deltaTime * 1f);

            CarEngineAudioSource.volume = CarSpeed;
            CarEngineAudioSource.pitch = NiceVibrationsDemoHelpers.Remap(CarSpeed, 0f, 1f, 0.5f, 1.25f);

            LeftWheel.Rotate(CarSpeed * Time.deltaTime * WheelRotationSpeed, Space.Self);
            RightWheel.Rotate(CarSpeed * Time.deltaTime * WheelRotationSpeed, Space.Self);

            _carPosition.x = _initialCarPosition.x + 0f;
            _carPosition.y = _initialCarPosition.y + 10 * CarSpeed * Mathf.PerlinNoise(Time.time * 10f, CarSpeed * 10f);
            _carPosition.z = 0f;
            CarBody.localPosition = _carPosition;

        }

        protected virtual void UpdateUI()
        {
            if (Knob.Active)
            {
                // start dent
                if (Time.time - _lastStartClickAt < StartClickDuration)
                {
                    float elapsedTime = StartClickCurve.Evaluate((Time.time - _lastStartClickAt) * (1 / StartClickDuration));
                    Knob._rectTransform.localScale = Vector3.one + Vector3.one * elapsedTime * 0.05f;
                    Knob._image.color = Color.Lerp(ActiveColor, Color.white, elapsedTime);
                }

                // other dents
                foreach (float f in Dents)
                {
                    if (((_knobValue >= f) && (_knobValueLastFrame < f)) || ((_knobValue <= f) && (_knobValueLastFrame > f)))
                    {
                        _lastDentAt = Time.time;
                        break;
                    }
                }
                if (Time.time - _lastDentAt < DentDuration)
                {
                    float elapsedTime = StartClickCurve.Evaluate((Time.time - _lastDentAt) * (1 / DentDuration));
                    Knob._rectTransform.localScale = Vector3.one + Vector3.one * elapsedTime * 0.02f;
                    Knob._image.color = Color.Lerp(ActiveColor, Color.white, elapsedTime * 0.05f);
                }
            }

            // gas bar
            PowerBar.UpdateBar(Power, 0f, MaximumPowerDuration);

            // power bars
            if (CarSpeed <= 0.1f)
            {
                for (int i = 0; i < SpeedBars.Count; i++)
                {
                    SpeedBars[i].SetActive(false);
                }
            }
            else
            {
                int barsAmount = (int)(CarSpeed * 5f);
                for (int i = 0; i < SpeedBars.Count; i++)
                {
                    if (i <= barsAmount)
                    {
                        SpeedBars[i].SetActive(true);
                    }
                    else
                    {
                        SpeedBars[i].SetActive(false);
                    }
                }
            }
        }
    }
}

