using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.EventSystems;

namespace Lofelt.NiceVibrations
{
    public class BallTouchZone : MonoBehaviour, IPointerExitHandler, IPointerEnterHandler
    {
        public RenderMode ParentCanvasRenderMode { get; protected set; }
        public RectTransform BallMover;
        protected bool _holding = false;
        protected PointerEventData _pointerEventData;
        protected Vector3 _newPosition;
        protected Canvas _canvas;
        protected float _initialZPosition;
        protected Vector2 _workPosition;


        protected virtual void Start()
        {
            Initialization();
        }

        protected virtual void Initialization()
        {
            ParentCanvasRenderMode = GetComponentInParent<Canvas>().renderMode;
            _canvas = GetComponentInParent<Canvas>();
            _initialZPosition = transform.position.z;
        }

        protected virtual void Update()
        {
            if (_holding)
            {
                _newPosition = GetWorldPosition(_pointerEventData.position);
            }
            else
            {
                _newPosition = Vector3.one * 5000f;
            }

            _newPosition.z = _initialZPosition;
            BallMover.position = _newPosition;
        }
        protected virtual Vector3 GetWorldPosition(Vector3 testPosition)
        {
            if (ParentCanvasRenderMode == RenderMode.ScreenSpaceCamera)
            {
                RectTransformUtility.ScreenPointToLocalPointInRectangle(_canvas.transform as RectTransform, testPosition, _canvas.worldCamera, out _workPosition);
                return _canvas.transform.TransformPoint(_workPosition);
            }
            else
            {
                return testPosition;
            }
        }

        public virtual void OnPointerEnter(PointerEventData data)
        {
            _holding = true;
            _pointerEventData = data;
        }

        public virtual void OnPointerExit(PointerEventData data)
        {
            _holding = false;
        }

    }
}
