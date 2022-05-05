using UnityEngine;
using UnityEngine.UI;
using System.Collections;
using Lofelt.NiceVibrations;

public class NiceVibrationsProfiling : MonoBehaviour
{
    public HapticSource haptic1Second;
    public HapticSource[] randomHaptics;
    public GameObject errorMessage;

    public void Start()
    {
        Screen.orientation = ScreenOrientation.LandscapeLeft;

        // Check if the device meets the Minimum requirements for Nice Vibrations and display an error if not.
        if (!DeviceCapabilities.meetsAdvancedRequirements)
        {
            errorMessage.SetActive(true);
        }
        else
        {
            errorMessage.SetActive(false);
        }
    }

    /// Runs a repeating test on a single second of haptics
    ///
    /// This test should not pass or fail, it's a perf test for manual measurement of
    /// impact of haptics on CPU, Memory and Energy efficiency.
    public void TestPerfOneSecond() {
        StartCoroutine(Play1SecondAndWait());
    }

    private IEnumerator Play1SecondAndWait()
    {
        // We don't set a fixed time for this measurement in code. While running the tests you
        // choose how long you want to measure for. So we loop infinintely here.
        while (true)
        {
            haptic1Second.Play();

            // Yield for over 1 second to let the actual haptic play and finish, with buffer.
            yield return new WaitForSeconds(1.2f);
        }
    }

    /// Play a:
    /// - random clip
    /// - random start time
    /// - random yield time
    ///
    /// This test should not pass or fail, it's a perf test for manual measurement of
    /// impact of haptics on CPU, Memory and Energy efficiency.
    public void TestPerfRandomClip() {
        StartCoroutine(PlayRandomAndWait());
    }

    private IEnumerator PlayRandomAndWait()
    {
        // We don't set a fixed time for this measurement in code. While running the tests you
        // choose how long you want to measure for. So we loop infinintely here.
        while (true)
        {
            int hapticChoice = Random.Range(0, randomHaptics.Length);

            // As we don't expose duration on HapticSource yet a fixed seek range is used here.
            // The haptics have been checked by hand for a duration that fits all.
            randomHaptics[hapticChoice].Seek(Random.Range(0.0f, 1.0f));
            randomHaptics[hapticChoice].Play();

            // Yield for a random duration to simulate game conditions.
            yield return new WaitForSeconds(Random.Range(0.5f, 5.0f));
        }
    }

    public void QuitButtonHandler()
    {
        Application.Quit();
    }
}
