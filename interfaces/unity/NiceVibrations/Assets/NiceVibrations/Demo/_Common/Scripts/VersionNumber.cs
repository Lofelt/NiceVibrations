// Copyright (c) Meta Platforms, Inc. and affiliates. 

using UnityEngine;
using UnityEngine.UI;

namespace Lofelt.NiceVibrations
{
    [RequireComponent(typeof(Text))]
    public class VersionNumber : MonoBehaviour
    {
        public string Version = "v3.3";

        protected Text _text;

        protected virtual void Awake()
        {
            _text = this.gameObject.GetComponent<Text>();
        }

        protected virtual void Start()
        {
            // There is not much space in the text, so make the string for alpha
            // and beta versions a bit shorter
            _text.text = Version.Replace("-alpha.", "a").Replace("-beta.", "b");

#if (UNITY_IOS && !UNITY_EDITOR)
            _text.text += " iOS " + DeviceCapabilities.platformVersion.ToString();
#elif (UNITY_ANDROID && !UNITY_EDITOR)
            _text.text += " Android " + DeviceCapabilities.platformVersion.ToString();
#endif
        }
    }
}
