// Copyright (c) Meta Platforms, Inc. and affiliates. 

using System.Collections;
using System.Collections.Generic;
using UnityEngine;

namespace Lofelt.NiceVibrations
{
    public class BallDemoWall : MonoBehaviour
    {
        protected RectTransform _rectTransform;
        protected BoxCollider2D _boxCollider2D;

        protected virtual void OnEnable()
        {
            _rectTransform = this.gameObject.GetComponent<RectTransform>();
            _boxCollider2D = this.gameObject.GetComponent<BoxCollider2D>();

            _boxCollider2D.size = new Vector2(_rectTransform.rect.size.x, _rectTransform.rect.size.y);
        }
    }
}
