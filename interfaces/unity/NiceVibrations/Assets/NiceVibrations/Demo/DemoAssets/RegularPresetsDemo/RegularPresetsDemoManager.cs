// Copyright (c) Meta Platforms, Inc. and affiliates. 

using System.Collections;
using UnityEngine;
using UnityEngine.UI;

namespace Lofelt.NiceVibrations
{
    public class RegularPresetsDemoManager : DemoManager
    {

        [Header("Image")]
        public Image IconImage;
        public Animator IconImageAnimator;

        [Header("Sprites")]
        public Sprite IdleSprite;

        public Sprite SelectionSprite;
        public Sprite SuccessSprite;
        public Sprite WarningSprite;
        public Sprite FailureSprite;
        public Sprite RigidSprite;
        public Sprite SoftSprite;
        public Sprite LightSprite;
        public Sprite MediumSprite;
        public Sprite HeavySprite;

        protected WaitForSeconds _turnDelay;
        protected WaitForSeconds _shakeDelay;
        protected int _idleAnimationParameter;

        protected virtual void Awake()
        {
            _turnDelay = new WaitForSeconds(0.02f);
            _shakeDelay = new WaitForSeconds(0.3f);
            _idleAnimationParameter = Animator.StringToHash("Idle");
            IconImageAnimator.SetBool(_idleAnimationParameter, true);
            IconImageAnimator.speed = 2f;
        }

        protected virtual void ChangeImage(Sprite newSprite)
        {
            StartCoroutine(ChangeImageCoroutine(newSprite));
        }

        protected virtual IEnumerator ChangeImageCoroutine(Sprite newSprite)
        {
            DebugAudioEmphasis.Play();
            IconImageAnimator.SetBool(_idleAnimationParameter, false);
            yield return _turnDelay;
            IconImage.sprite = newSprite;
            yield return _shakeDelay;
            IconImageAnimator.SetBool(_idleAnimationParameter, true);
            yield return _turnDelay;
            IconImage.sprite = IdleSprite;
        }

        public virtual void SelectionButton()
        {
            HapticPatterns.PlayPreset(HapticPatterns.PresetType.Selection);
            ChangeImage(SelectionSprite);
        }

        public virtual void SuccessButton()
        {
            HapticPatterns.PlayPreset(HapticPatterns.PresetType.Success);
            ChangeImage(SuccessSprite);
        }

        public virtual void WarningButton()
        {
            HapticPatterns.PlayPreset(HapticPatterns.PresetType.Warning);
            ChangeImage(WarningSprite);
        }

        public virtual void FailureButton()
        {
            HapticPatterns.PlayPreset(HapticPatterns.PresetType.Failure);
            ChangeImage(FailureSprite);
        }

        public virtual void RigidButton()
        {
            HapticPatterns.PlayPreset(HapticPatterns.PresetType.RigidImpact);
            ChangeImage(RigidSprite);
        }

        public virtual void SoftButton()
        {
            HapticPatterns.PlayPreset(HapticPatterns.PresetType.SoftImpact);
            ChangeImage(SoftSprite);
        }

        public virtual void LightButton()
        {
            HapticPatterns.PlayPreset(HapticPatterns.PresetType.LightImpact);
            ChangeImage(LightSprite);
        }

        public virtual void MediumButton()
        {
            HapticPatterns.PlayPreset(HapticPatterns.PresetType.MediumImpact);
            ChangeImage(MediumSprite);
        }

        public virtual void HeavyButton()
        {
            HapticPatterns.PlayPreset(HapticPatterns.PresetType.HeavyImpact);
            ChangeImage(HeavySprite);
        }
    }
}
