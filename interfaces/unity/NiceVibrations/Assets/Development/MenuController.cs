using UnityEngine;
using UnityEngine.UI;
using Lofelt.NiceVibrations;

namespace Lofelt.NiceVibrations
{
    public class MenuController : MonoBehaviour
    {
        public HapticSource hapticA;
        public AudioSource audioA;
        public HapticSource hapticB;
        public AudioSource audioB;
        public HapticClip Achievement_1;
        public HapticClip Achievement_2;
        public HapticClip Achievement_3;
        public Image AchieveButton1;
        public Image AchieveButton2;
        public Image AchieveButton3;
        public Slider AchievementSlider;
        private float achievementDuration = 2.296f; // seconds
        public Image HighButton;
        public Image MatchedButton;
        public Image LowButton;
        public Slider clipLevelSlider;
        public Text clipLevelSliderText;
        public Slider frequencyShiftSlider;
        public Text frequencyShiftSliderText;

        public Slider outputLevelSlider;
        public Text outputLevelSliderText;

        public Text deviceCapabilitiesText;

        public void Start()
        {
            Screen.orientation = ScreenOrientation.LandscapeLeft;
            
            // Default to Achievement_1
            hapticA.clip = Achievement_1;
            AchieveButton1.color = new Color32(255, 67, 56, 255);
            AchieveButton2.color = new Color32(255, 255, 255, 255);
            AchieveButton3.color = new Color32(255, 255, 255, 255);

            // Default to A > B voice priority
            hapticA.priority = 128;
            hapticB.priority = 255;
            HighButton.color = new Color32(255, 67, 56, 255);
            MatchedButton.color = new Color32(255, 255, 255, 255);
            LowButton.color = new Color32(255, 255, 255, 255);

            // Show Device Capabilities
            deviceCapabilitiesText.text = "Platform: " + DeviceCapabilities.platform.ToString() + "\n";
            deviceCapabilitiesText.text += "Platform Version: " + DeviceCapabilities.platformVersion.ToString() + "\n";
            deviceCapabilitiesText.text += "Version Supported: " + DeviceCapabilities.isVersionSupported.ToString() + "\n"; 
            deviceCapabilitiesText.text += "Advanced Requirements: " + DeviceCapabilities.meetsAdvancedRequirements.ToString() + "\n";
            deviceCapabilitiesText.text += "Amplitude Control: " + DeviceCapabilities.hasAmplitudeControl.ToString() + "\n";
            deviceCapabilitiesText.text += "Frequency Control: " + DeviceCapabilities.hasFrequencyControl.ToString() + "\n";
            deviceCapabilitiesText.text += "Amplitude Modulation: " + DeviceCapabilities.hasAmplitudeModulation.ToString() + "\n";
            deviceCapabilitiesText.text += "Frequency Modulation: " + DeviceCapabilities.hasFrequencyModulation.ToString() + "\n";
            deviceCapabilitiesText.text += "Emphasis: " + DeviceCapabilities.hasEmphasis.ToString() + "\n";
            deviceCapabilitiesText.text += "Emphasis Emulation: " + DeviceCapabilities.canEmulateEmphasis.ToString() + "\n";
            deviceCapabilitiesText.text += "Looping: " + DeviceCapabilities.canLoop.ToString() + "\n";
        }

        public void SelectAchievement1Handler()
        {
            hapticA.clip = Achievement_1;
            AchieveButton1.color = new Color32(255, 67, 56, 255);
            AchieveButton2.color = new Color32(255, 255, 255, 255);
            AchieveButton3.color = new Color32(255, 255, 255, 255);
        }

        public void SelectAchievement2Handler()
        {
            hapticA.clip = Achievement_2;
            AchieveButton1.color = new Color32(255, 255, 255, 255);
            AchieveButton2.color = new Color32(255, 67, 56, 255);
            AchieveButton3.color = new Color32(255, 255, 255, 255);

        }

        public void SelectAchievement3Handler()
        {
            hapticA.clip = Achievement_3;
            AchieveButton1.color = new Color32(255, 255, 255, 255);
            AchieveButton2.color = new Color32(255, 255, 255, 255);
            AchieveButton3.color = new Color32(255, 67, 56, 255);
        }

        public void PlayAchievementHandler()
        {
            audioA.Play();
            hapticA.Play();
        }

        public void StopAchievementHandler()
        {
            audioA.Stop();
            hapticA.Stop();
        }

        public void SeekAchievementHandler()
        {
            // Stop audio+haptics, otherwise while playing and setting a new seek value
            // it will continue to play audio from the new time position
            audioA.Stop();
            hapticA.Stop();

            float achievementSeekTimeValue = AchievementSlider.value * achievementDuration;
            hapticA.Seek(achievementSeekTimeValue);
            audioA.time = achievementSeekTimeValue;
        }

        public void HighPriorityHandler()
        {
            hapticA.priority = 128;
            hapticB.priority = 255;
            HighButton.color = new Color32(255, 67, 56, 255);
            MatchedButton.color = new Color32(255, 255, 255, 255);
            LowButton.color = new Color32(255, 255, 255, 255);
        }

        public void LowPriorityHandler()
        {
            hapticA.priority = 128;
            hapticB.priority = 0;
            HighButton.color = new Color32(255, 255, 255, 255);
            MatchedButton.color = new Color32(255, 255, 255, 255);
            LowButton.color = new Color32(255, 67, 56, 255);
        }

        public void MatchedPriorityHandler()
        {
            hapticA.priority = 128;
            hapticB.priority = 128;
            HighButton.color = new Color32(255, 255, 255, 255);
            MatchedButton.color = new Color32(255, 67, 56, 255);
            LowButton.color = new Color32(255, 255, 255, 255);
        }

        public void UpdateClipLevelSliderText()
        {
            clipLevelSliderText.text = clipLevelSlider.value.ToString("0.0");
        }

        public void UpdateFrequencyShiftSliderText()
        {
            frequencyShiftSliderText.text = frequencyShiftSlider.value.ToString("0.0");
        }

        public void UpdateOutputLevelSliderText()
        {
            outputLevelSliderText.text = outputLevelSlider.value.ToString("0.0");
        }

        public void PlayStrokeHandler()
        {
            audioB.Play();
            hapticB.Play();
        }

        public void QuitButtonHandler()
        {
            Application.Quit();
        }
    }
}
