// Copyright (c) Meta Platforms, Inc. and affiliates. 

using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.EventSystems;

namespace Lofelt.NiceVibrations
{
    public class MMUIShaker : MonoBehaviour
    {
        public float Amplitude;
        public float Frequency;
        public bool Shaking = false;

        protected Vector3 _initialPosition;
        protected Vector3 _shakePosition;
        protected RectTransform _rectTransform;

        protected virtual void Start()
        {
            _rectTransform = this.gameObject.GetComponent<RectTransform>();
            _initialPosition = _rectTransform.localPosition;
        }

        public virtual IEnumerator Shake(float duration)
        {
            Shaking = true;
            yield return new WaitForSeconds(duration);
            Shaking = false;
        }

        protected virtual void Update()
        {
            if (!Shaking)
            {
                _rectTransform.localPosition = _initialPosition;
                return;
            }
            else
            {
                _shakePosition.x = Mathf.PerlinNoise(-(Time.time) * Frequency, Time.time * Frequency) * Amplitude - Amplitude / 2f;
                _shakePosition.y = Mathf.PerlinNoise(-(Time.time + 0.25f) * Frequency, Time.time * Frequency) * Amplitude - Amplitude / 2f;
                _shakePosition.z = Mathf.PerlinNoise(-(Time.time + 0.5f) * Frequency, Time.time * Frequency) * Amplitude - Amplitude / 2f;
                _rectTransform.localPosition = _initialPosition + _shakePosition;
            }
        }
    }
}
