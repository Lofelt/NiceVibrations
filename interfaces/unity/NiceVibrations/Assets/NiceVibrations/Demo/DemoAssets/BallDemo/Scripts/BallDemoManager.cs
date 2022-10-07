// Copyright (c) Meta Platforms, Inc. and affiliates. 

using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;

namespace Lofelt.NiceVibrations
{
    public class BallDemoManager : DemoManager
    {

        [Header("Ball")]
        public Vector2 Gravity = new Vector2(0, -30f);

        protected virtual void Start()
        {
            Physics2D.gravity = Gravity;
        }
    }
}
