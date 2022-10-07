// Copyright (c) Meta Platforms, Inc. and affiliates. 

using System.Collections;
using System.Collections.Generic;
using UnityEngine;

namespace Lofelt.NiceVibrations
{
    public class BallPusher : MonoBehaviour
    {
        public float Force = 5f;
        public BallDemoBall TargetBall;
        protected Vector2 _direction;

        protected virtual void OnTriggerEnter2D(Collider2D collider)
        {
            if (collider.gameObject != TargetBall.gameObject)
            {
                return;
            }

            _direction = (collider.transform.position - this.transform.position).normalized;
            _direction.y = 1f;
            collider.attachedRigidbody.velocity = Vector2.zero;
            collider.attachedRigidbody.AddForce(_direction * Force);
            TargetBall.HitPusher();
        }
    }
}
