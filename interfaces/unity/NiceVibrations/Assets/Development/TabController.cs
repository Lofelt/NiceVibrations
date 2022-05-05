using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;

namespace Lofelt.NiceVibrations
{
    public class TabController : MonoBehaviour
    {
        public RectTransform generalPanel;
        public RectTransform modulationPanel;
        public RectTransform settingsPanel;

        public void switchToModulationPanel()
        {
            generalPanel.gameObject.SetActive(false);
            modulationPanel.gameObject.SetActive(true);
            settingsPanel.gameObject.SetActive(false);
            generalPanel.anchoredPosition = new Vector2(-2000, 0);
            modulationPanel.anchoredPosition = new Vector2(0, 0);
            settingsPanel.anchoredPosition = new Vector2(-2000, 0);
        }

        public void switchToGeneralPanel()
        {
            generalPanel.gameObject.SetActive(true);
            modulationPanel.gameObject.SetActive(false);
            settingsPanel.gameObject.SetActive(false);
            generalPanel.anchoredPosition = new Vector2(0, 0);
            modulationPanel.anchoredPosition = new Vector2(-2000, 0);
            settingsPanel.anchoredPosition = new Vector2(-2000, 0);
        }

        public void switchToSettingsPanel()
        {
            generalPanel.gameObject.SetActive(false);
            modulationPanel.gameObject.SetActive(false);
            settingsPanel.gameObject.SetActive(true);
            generalPanel.anchoredPosition = new Vector2(-2000, 0);
            modulationPanel.anchoredPosition = new Vector2(-2000, 0);
            settingsPanel.anchoredPosition = new Vector2(0, 0);
        }
    }
}
