
// (c) Meta Platforms, Inc. and affiliates. Confidential and proprietary.import XCTest
import LofeltHaptics
import AVFoundation
import os.log

@testable import LofeltHapticsTestApp

/*!
 @class         LofeltHapticsTestAppTests

 @brief         The LofeltHapticsTestAppTests class

 @discussion    These are our semi-automated integration tests for iOS. In order to run these
                tests you need an iPhone with iOS 13 or later. Some of the tests take a long
                time to run so consider whether you want to test one specific test case or all
                of them. Also, if you want to run the performance tests you need to enable
                enablePerformanceTests below. The manual part of testing is for you to confirm
                that what is printed in the console actually happens on the iPhone. Whether the
                test passes or fails is both down to the automated test result and your manual
                test result.

                In case you didn't know: The Test navigator (⌘ + 6) gives you all tests in a nice list
                and you can click the button to the right of the one you would like to run.

                Happy testing!
 @author        James Kneafsey
 @copyright     © 2020 Lofelt. All rights reserved.
 @version
 */
class LofeltHapticsTestAppTests: XCTestCase {
    // Global variable to enable/disable performance tests.
    let enablePerformanceTests = false;

    override func setUp() {
        // Put setup code here. This method is called before the invocation of each test method in the class.
    }

    override func tearDown() {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }

    /// Plays a clip.
    func testPlay() {
        printInstructions(testName: "testPlay",
                          instructions: "Confirm you feel an impact.")

        playHaptic(assetName: "Impact_1", playDuration: 2, printTimings: false)
    }

    /// Plays and stops a clip.
    func testPlayAndStop() {
        printInstructions(testName: "testPlayAndStop",
                          instructions: "Confirm you feel a haptic for 2 seconds and then nothing.")

        playHaptic(assetName: "OP-Z", playDuration: 2, printTimings: false, stopHaptic: true)
        print("Should feel nothing now for 2 seconds")

        // Wait a bit to confirm that the haptic has stopped while we are still running.
        Thread.sleep(forTimeInterval: 2)
    }

    /// Plays a haptic file in the old version 0.2 format AKA vij.
    func testPlayAndStopVij() {
        printInstructions(testName: "testPlayAndStopVij",
                          instructions: "Confirm that you feel the revving of a car engine.")

        playHaptic(assetName: "vij_car", playDuration: 4, printTimings: false, stopHaptic: true)
    }

    /// Tests that loading a .haptic clip with loadFromData works
    func testLoadFromData() {
        printInstructions(testName: "testLoadFromData",
                          instructions: "Confirm that you feel some haptics playing for one second.")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "1second")
        try! haptics.load(from: hapticData!.data)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.0)
    }

    /// Tests that nothing plays after an invalid haptic is loaded, not even a previously-loaded valid haptic.
    func testInvalidHaptic() {
        printInstructions(testName: "testInvalidHaptic",
                          instructions: "Confirm that you feel nothing.")

        let haptics = try! LofeltHaptics.init()

        // First load a valid haptic.
        var hapticData = NSDataAsset(name: "OP-Z")
        var dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        // Then load an invalid haptic.
        hapticData = NSDataAsset(name: "invalid_v0_car")
        dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)

        if (try? haptics.load(dataString! as String)) != nil {
            assert(false, "Load should have failed as this haptic is invalid.")
        }

        if (try? haptics.play()) != nil {
            assert(false, "Play should have failed as this haptic is invalid.")
        }
    }

    /// Plays a 37-second clip.
    func testLongPlay() {
        printInstructions(testName: "testLongPlay",
                          instructions: "Confirm that you feel a haptic playing for longer than 30 seconds")

        playHaptic(assetName: "thirty-five-seconds", playDuration: 40)
    }

    /// Plays the same clip 5 times in a row up to a total playing time exceeding 30 seconds.
    /// This is to make sure one instance of LofeltHaptics keeps playing clips past 30 seconds.
    func testLongPlayOfMultipleClips() {
        printInstructions(testName: "test5LongPlays",
                          instructions: "Confirm that you see the message 'Played 5 times' after 5 complete plays have occurred")

        let haptics = try! LofeltHaptics.init()

        for i in 1...5 {
            playHaptic(assetName: "OP-Z", playDuration: 10, stopHaptic: true, haptics: haptics)

            print("Played \(i) time(s)")
        }
    }

    /// Loads and plays 3 different clips on the same LofeltHaptics instance.
    func test3DifferentClips() {
        printInstructions(testName: "test3DifferentClips",
                          instructions: "Confirm that you feel 3 different clips play for 2 seconds each with 2-second pauses in between")

        let haptics = try! LofeltHaptics.init()

        print("Clip 1")

        playHaptic(assetName: "OP-Z", playDuration: 2, stopHaptic: true, haptics: haptics)

        print("No output")

        // Leave a gap in between clips.
        Thread.sleep(forTimeInterval: 2)

        print("Clip 2")

        playHaptic(assetName: "thirty-five-seconds", playDuration: 2, stopHaptic: true, haptics: haptics)

        print("No output")

        // Leave a gap in between clips.
        Thread.sleep(forTimeInterval: 2)

        print("Clip 3")

        playHaptic(assetName: "emphasis-and-continuous", playDuration: 2, stopHaptic: true, haptics: haptics)

        print("Test complete")
    }

    /// Plays a clip with 6 emphases.
    func testPlayEmphasis() {
        printInstructions(testName: "testPlayEmphasis",
                          instructions: "Confirm that you feel 6 transients increasing in pitch")

        playHaptic(assetName: "emphasis", playDuration: 2.7)
    }

    /// Plays a clip with 6 emphases and a continuous envelope decreasing in frequency.
    func testPlayEmphasisAndContinuous() {
        printInstructions(testName: "testPlayEmphasisAndContinuous",
                          instructions: "Confirm that you feel 6 transients increasing in sharpness"
                            + " and a continuous envelope decreasing from maximum to minimum sharpness")

        playHaptic(assetName: "emphasis-and-continuous", playDuration: 3.3)
    }

    /// Plays a clip with amplitude and frequency ramping up and down.
    func testPlayRamp() {
        printInstructions(testName: "testPlayRamp",
                          instructions: "Confirm that you can feel the amplitude fade in at the start and end,"
                          + " and the frequency going from minimum to maximum and back during full amplitude")

        playHaptic(assetName: "ramp", playDuration: 4.5)
    }

    /// Tests that amplitude always returns to zero after a clip completes playing.
    func testZeroAmplitudeAfterHapticFinishes() {
        printInstructions(testName: "testZeroAmplitudeAfterHapticFinishes",
                          instructions: "Confirm that feel a constant maximum amplitude for 2 seconds and then nothing for 2 seconds")

        playHaptic(assetName: "constant-amplitude", playDuration: 2)

        print("Should feel nothing now for 2 seconds")

        // Wait a bit to confirm that amplitude has dropped to zero even while we are still running.
        Thread.sleep(forTimeInterval: 2)
    }

    /// Tests seeking forward and backward in a clip with varying amplitude
    func testSeekAmplitude() {
        printInstructions(testName: "testSeekAmplitude",
                          instructions: "Confirm that you feel the following: \n" +
                                        "1. After 2 seconds: Amplitude goes to 0 and back to 1 within 3 seconds\n" +
                                        "2. After 1 seconds: Amplitude goes to 0 and back to 1 within 3 seconds\n" +
                                        "3. After 0 seconds: Amplitude jumps to a low value and then fades out within 1 second")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "seek")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        try! haptics.play()

        Thread.sleep(forTimeInterval: 5.0)
        try! haptics.seek(1.0)
        Thread.sleep(forTimeInterval: 4.0)
        try! haptics.seek(9.0)
        Thread.sleep(forTimeInterval: 5.0)
    }

    /// Like testSeekAmplitude(), but in a clip with varying frequency
    func testSeekFrequency() {
        printInstructions(testName: "testSeekFrequency",
                          instructions: "Confirm that you feel the following: \n" +
                                        "1. After 2 seconds: Frequency goes to 0 and back to 1 within 3 seconds\n" +
                                        "2. After 1 seconds: Frequency goes to 0 and back to 1 within 3 seconds\n" +
                                        "3. After 0 seconds: Frequency jumps to a low value and then fades to 0 within 1 second")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "seek_frequency")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        try! haptics.play()

        Thread.sleep(forTimeInterval: 5.0)
        try! haptics.seek(1.0)
        Thread.sleep(forTimeInterval: 4.0)
        try! haptics.seek(9.0)
        Thread.sleep(forTimeInterval: 5.0)
    }

    /// Tests seeking to a negative time during playback
    func testSeekNegativeDuringPlayback() {
        printInstructions(testName: "testSeekNegativeDuringPlayback",
                          instructions: "Confirm that you feel the following: \n" +
                                        "1. A ramp starts descending from 1\n" +
                                        "2. After 2 seconds: a gap in playback, zero amplitude\n" +
                                        "3. After 2 seconds: The ramp restarts from 1 and descends to 0")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "seek")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        try! haptics.play()

        Thread.sleep(forTimeInterval: 2.0)
        try! haptics.seek(-2.0)
        Thread.sleep(forTimeInterval: 5.0)
    }

    /// Tests seeking without calling play() before
    func testSeekWithoutPlay() {
        printInstructions(testName: "testSeekWithoutPlay",
                          instructions: "Confirm that you feel the amplitude ramping down from 1 to 0 within 5 seconds")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "seek")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.seek(5.0)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 8.0)
    }

    /// Tests seeking without calling play() before
    func testSeekWithoutPlayFromNegativeSeekTime() {
        printInstructions(testName: "testSeekWithoutPlayFromNegativeSeekTime",
                          instructions: "Confirm that you feel the amplitude ramping down from 1 to 0 within 5 seconds, after an initial delay of 2 seconds")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "seek")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.seek(-2.0)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 5.0)
    }

    /// Tests seeking from within a large gap between two breakpoints
    func testSeekFromBreakpointGap() {
        printInstructions(testName: "testSeekFromBreakpointGap",
                          instructions: "Confirm that you feel the following:\n" +
                                        "1. Amplitude ramping down from 1 to 0.5 within 2.5 seconds\n" +
                                        "2. Amplitude jumping back to 1\n" +
                                        "3. Amplitude ramping down from 1 to 0 withing 5 seconds")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "seek")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.seek(5.0)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 2.5)
        try! haptics.seek(5.0)
        Thread.sleep(forTimeInterval: 8)
    }

    /// Tests that amplitude multiplication is applied correctly
    func testAmplitudeMultiplication() {
        printInstructions(testName: "testAmplitudeMultiplication",
                          instructions: "Confirm that you feel the following:\n" +
                                        "1. A clip plays for 1 second\n" +
                                        "2. Pause of 500ms\n" +
                                        "3. The same clip plays for 1 second again, with much lesser intensity")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "1second")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.5)
        try! haptics.setAmplitudeMultiplication(0.25)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.0)
    }

    /// Same as testAmplitudeMultiplication(), but for frequency shift instead
    /// of amplitude mulitplication
    func testFrequencyShift() {
        printInstructions(testName: "testFrequencyShift",
                          instructions: "Confirm that you feel the following:\n" +
                                        "1. A clip plays for 1 second\n" +
                                        "2. Pause of 500ms\n" +
                                        "3. The same clip plays for 1 second again, with a shifted frequency")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "1second")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.5)
        try! haptics.setFrequencyShift(0.25)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.0)
    }

    /// Tests that amplitude multiplication is applied correctly while a clip is playing
    func testAmplitudeMultiplicationLive() {
        printInstructions(testName: "testAmplitudeMultiplicationLive",
                          instructions: "Confirm that you feel the following:\n" +
                                        "1. A clip with a constant amplitude and frequency plays for 1 second\n" +
                                        "2. The amplitude changes to half the value\n" +
                                        "3. The clip continues for 1 second more")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "constant-amplitude")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.0)
        try! haptics.setAmplitudeMultiplication(0.5)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.0)
    }

    /// Same as testAmplitudeMultiplicationLive(), but for frequency shift
    /// instead of amplitude mulitplication
    func testFrequencyShiftLive() {
        printInstructions(testName: "testFrequencyShiftLive",
                          instructions: "Confirm that you feel the following:\n" +
                                        "1. A clip with a constant amplitude and frequency plays for 1 second\n" +
                                        "2. The frequency is lowered\n" +
                                        "3. The clip continues for 1 second more")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "constant-amplitude")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.0)
        try! haptics.setFrequencyShift(-0.3)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.0)
    }

    /// Tests that amplitude multiplication of factor 1.0 is applied correctly while
    /// a clip is playing.
    /// A factor of 1.0 should make no detectable difference.
    func testAmplitudeMultiplicationLiveNoChange() {
        printInstructions(testName: "testAmplitudeMultiplicationLiveNoChange",
                          instructions: "Confirm that you feel the following:\n" +
                                        "1. A clip with a constant amplitude and frequency plays for 2 seconds")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "constant-amplitude")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.0)
        try! haptics.setAmplitudeMultiplication(1.0)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.0)
    }

    /// Same as testAmplitudeMultiplicationLiveNoChange(), but for frequency
    /// shift instead of amplitude multiplication.
    /// A shift of 0.0 should make no detectable difference.
    func testFrequencyShiftLiveNoChange() {
        printInstructions(testName: "testFrequencyShiftLiveNoChange",
                          instructions: "Confirm that you feel the following:\n" +
                                        "1. A clip with a constant amplitude and frequency plays for 2 seconds")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "constant-amplitude")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.0)
        try! haptics.setFrequencyShift(0.0)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 1.0)
    }

    /// Tests that a constantly changed amplitude multiplication works correctly.
    func testAmplitudeMultiplicationLiveRamp() {
        printInstructions(testName: "testAmplitudeMultiplicationLiveRamp",
                          instructions: "Confirm that you feel the following:\n" +
                                        "1. The amplitude ramps up from 0 to 1 within 2 seconds")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "constant-amplitude")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        try! haptics.play()
        let iterations = 100;
        for i in 1...iterations {
            try! haptics.setAmplitudeMultiplication(Float(i) * (1.0 / (Float(iterations))))
            Thread.sleep(forTimeInterval: 2.0 / Double(iterations))
        }
    }

    /// Same as testAmplitudeMultiplicationLiveRamp(), but for frequency shift
    /// instead of amplitude multiplication
    func testFrequencyShiftLiveRamp() {
        printInstructions(testName: "testFrequencyShiftLiveRamp",
                          instructions: "Confirm that you feel the following:\n" +
                                        "1. The frequency ramps up from 0 to 1 within 2 seconds")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "constant-amplitude")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        try! haptics.play()
        let iterations = 100;
        for i in 1...iterations {
            try! haptics.setFrequencyShift(((Float(i) - (Float(iterations) / 2.0)) / (Float(iterations) / 2.0)) / 2.0)
            Thread.sleep(forTimeInterval: 2.0 / Double(iterations))
        }
    }

    /// Tests if the duration of the loaded clip
    func testGetClipDuration() {
        printInstructions(testName: "testGetClipDuration",
                          instructions: "Confirm that the test passes")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "OP-Z")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)
        let expectedDuration: Float = 9.804535
        let resultDuration: Float = haptics.getClipDuration()

        assert(expectedDuration.isEqual(to: resultDuration))
    }

    /// Tests if enabling loop makes the playback of the clip repeat more than one time
    func testLoopClip() {
        printInstructions(testName: "testLoopClip",
                          instructions: "Confirm that the clip playback is repeated")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "Impact_1")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.loop(true)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 8.0)

    }

    /// Tests if enabling loop makes the playback of single-continuous-amplitude-only clip repeat more
    /// than one time without any drops in the vibration. It should feel like a single "vibration"
    func testLoopContinuousSignalClip() {
        printInstructions(testName: "testLoopContinuousSignalClip",
                          instructions: "Confirm that a single intensity vibration is felt and " +
                          "that are no drops")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "constant-amplitude")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.loop(true)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 8.0)

    }

    // Tests enabling loop for ramp-up/ramp-down clips
    func testLoopRampUpAndRampDownClips() {
        printInstructions(testName: "testLoopRampUpAndRampDownClips",
                          instructions: "Confirm that repetition is felt for:\n" +
                          "1. Ramp-up clip for the first 8 seconds\n" +
                          "2. Ramp-down clip for the last 8 seconds")

        let haptics = try! LofeltHaptics.init()
        var hapticData = NSDataAsset(name: "ramp_up")
        var dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.loop(true)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 8.0)
        try! haptics.stop()

        Thread.sleep(forTimeInterval: 2.0)

        hapticData = NSDataAsset(name: "ramp_down")
        dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.loop(true)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 8.0)


    }

    /// Tests enabling loop for ramp-up/ramp-down clips. An emphasis breakpoint should be felt
    /// along with every repetition
    func testLoopRampUpAndRampDownWithEmphasisClips() {

        printInstructions(testName: "testLoopRampUpAndRampDownWithEmphasisClips",
                          instructions: "Confirm that repetition is felt for:\n" +
                          "1. Ramp-up clip with 1 emphasis at every repetition for 8 seconds\n" +
                          "2. Ramp-down clip with 1 emphasis at every repetition for 8 seconds")

        let haptics = try! LofeltHaptics.init()
        var hapticData = NSDataAsset(name: "ramp_up_with_emp")
        var dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.loop(true)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 8.0)
        try! haptics.stop()

        Thread.sleep(forTimeInterval: 2.0)

        hapticData = NSDataAsset(name: "ramp_down_with_emp")
        dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.loop(true)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 8.0)


    }
    // Tests enabling looping for a emphasis-only-clip
    func testLoopEmphasisOnlyClip() {
        printInstructions(testName: "testLoopEmphasisOnlyClip",
                          instructions: "Confirm that a emphasis-only-clip is repeating")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "emphasis")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.loop(true)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 8.0)

    }

    /// Tests enabling loop for frequency ramp-up/ramp-down clips.
    func testLoopContinuousClipFrequencyRampUpAndRampDown() {
        printInstructions(testName: "testLoopContinuousClipFrequencyRampUpAndRampDown",
                          instructions: "Confirm that repetition is felt for:\n" +
                          "1. Frequency ramp-up clip for the first 8 seconds\n" +
                          "2. Frequency ramp-down clip for the last 8 seconds")

        let haptics = try! LofeltHaptics.init()
        var hapticData = NSDataAsset(name: "freq_ramp_up")
        var dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.loop(true)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 8.0)
        try! haptics.stop()

        Thread.sleep(forTimeInterval: 2.0)

        hapticData = NSDataAsset(name: "freq_ramp_down")
        dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.loop(true)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 8.0)

    }

    /// Tests if enabling loop makes the playback of the clip repeat more than one time
    func testLoopClipAfterSeekingAfterEnd() {
        printInstructions(testName: "testLoopClip",
                          instructions: "Confirm that the clip playback is repeated")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "Impact_1")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        try! haptics.loop(true)
        try! haptics.play()
        Thread.sleep(forTimeInterval: 0.5)
        try! haptics.seek(10.0)
        Thread.sleep(forTimeInterval: 7.5)
    }

    func testRapidTransients()
    {
        printInstructions(testName: "testRapidTransients",
                          instructions: "Confirm that all transients feel the same and none of them glitch")

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "emphasis-only")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        for _ in 1...30 {
            try! haptics.play()
            Thread.sleep(forTimeInterval: 0.2)
        }
    }


    /// -------------------------------------------------------------------------------------------
    /// Performance tests
    /// -------------------------------------------------------------------------------------------


    /// Runs a 5 minute test on a single second of haptics
    ///
    /// This test should not pass or fail, it's a perf test for manual measurement of
    /// impact of haptics on CPU, Memory and Energy efficiency.
    func testPerfOneSecondForFiveMinutes() throws {
        try XCTSkipUnless(enablePerformanceTests);

        let haptics = try! LofeltHaptics.init()
        var hapticData: NSDataAsset!
        var dataString: NSString!

        // Sleep for over 1 second to let the actual haptic play and finish, with buffer.
        let timeForHapticToPlay = 1.2
        let fiveMinutes = Int((60 / timeForHapticToPlay) * 5)
        for _ in 0...fiveMinutes{
            autoreleasepool {
                hapticData = NSDataAsset(name: "1second")
                dataString = NSString(data: hapticData!.data , encoding: String.Encoding.utf8.rawValue)

                try! haptics.load(dataString! as String);
                try! haptics.play()
                // Let the engine play.
                Thread.sleep(forTimeInterval: timeForHapticToPlay)
            }
        }
    }

    /// Seeks 2500000 times to a random location in a long clip.
    ///
    /// Like testPerfOneSecondForFiveMinutes(), this is a perf test that should be run manually
    /// for profiling e.g. CPU and memory usage in Instruments.
    func testSeekPerformance() throws {
        try XCTSkipUnless(enablePerformanceTests);

        let haptics = try! LofeltHaptics.init()
        let hapticData = NSDataAsset(name: "long_clip")
        let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
        try! haptics.load(dataString! as String)

        // Create a point of interest region for the seek duration, so that the seeking can
        // clearly be differentiated from the loading phase in Instruments
        let pointsOfInterest = OSLog(subsystem: Bundle.main.bundleIdentifier!, category: .pointsOfInterest)
        let signpost_id = OSSignpostID(log: pointsOfInterest)
        os_signpost(.begin, log: pointsOfInterest, name: "Seeking", signpostID: signpost_id)
        print("Starting to seek...")

        for _ in 1...2500000 {
            try! haptics.seek(Float.random(in: 0...30))
        }

        print("Done seeking")
        os_signpost(.end, log: pointsOfInterest, name: "Seeking", signpostID: signpost_id)

        // Let the main thread sleep for a bit, in case the streaming thread needs some time to
        // finish queued seek commands
        Thread.sleep(forTimeInterval: 5)
    }
}

