using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;

namespace Lofelt.NiceVibrations
{
    /// <summary>
    /// A minimal, demo only class, used to rotate an image in the demo's UI
    /// </summary>
    public class HapticClipsDemoRotator : MonoBehaviour
    {
        /// the speed at which the image should rotate
        public Vector3 RotationSpeed = new Vector3(0, 0, 100f);

        /// <summary>
        /// On Update we rotate our image
        /// </summary>
        protected void Update()
        {
            this.transform.Rotate(RotationSpeed * Time.deltaTime, Space.Self);
        }
    }
}
