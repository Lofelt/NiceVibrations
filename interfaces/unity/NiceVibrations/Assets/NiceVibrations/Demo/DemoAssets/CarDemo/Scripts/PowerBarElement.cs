using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;

namespace Lofelt.NiceVibrations
{
    public class PowerBarElement : MonoBehaviour
    {
        public float BumpDuration = 0.15f;
        public Color NormalColor;
        public Color InactiveColor;
        public AnimationCurve Curve;

        protected Image _image;
        protected float _bumpDuration = 0f;
        protected bool _active = false;
        protected bool _activeLastFrame = false;

        protected virtual void Awake()
        {
            _image = this.gameObject.GetComponent<Image>();
        }

        public virtual void SetActive(bool status)
        {
            _active = status;
            _image.color = status ? NormalColor : InactiveColor;
        }

        protected virtual void Update()
        {
            if (_active && !_activeLastFrame)
            {
                StartCoroutine(ColorBump());
            }
            _activeLastFrame = _active;
        }

        protected virtual IEnumerator ColorBump()
        {
            _bumpDuration = 0f;
            while (_bumpDuration < BumpDuration)
            {
                float curveValue = Curve.Evaluate(_bumpDuration / BumpDuration);
                _image.color = Color.Lerp(NormalColor, Color.white, curveValue);

                _bumpDuration += Time.deltaTime;
                yield return null;
            }

            _image.color = NormalColor;
        }
    }
}
