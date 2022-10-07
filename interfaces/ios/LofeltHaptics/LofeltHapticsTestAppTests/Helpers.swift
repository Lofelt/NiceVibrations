
// (c) Meta Platforms, Inc. and affiliates. Confidential and proprietary.import Foundation
import LofeltHaptics
import AVFoundation
import UIKit

/// Convenience function for haptic playback tests.
///
/// ```
/// playHaptic(assetName: "my-haptic", duration: 4)
/// ```
///
/// - Parameters:
///   - assetName: The name of the haptic in the asset catalog.
///   - duration: The number of seconds to allow it to play out.
///   - printTimestamps: Whether the timings of operations should be printed.
///   - stopHaptic: Whether the haptic should be stopped after playing for `duration`.
func playHaptic(assetName:String, playDuration:TimeInterval, printTimings:Bool = false, stopHaptic:Bool = false, haptics:LofeltHaptics = try! LofeltHaptics.init()) {
    let hapticData = NSDataAsset(name: assetName)
    let dataString = NSString(data: hapticData!.data, encoding: String.Encoding.utf8.rawValue)
    var timeBefore = Date().timeIntervalSince1970

    try! haptics.load(dataString! as String)

    if (printTimings) {
        print("Load duration: ", String(format:"%f", Date().timeIntervalSince1970 - timeBefore), "s")
        timeBefore = Date().timeIntervalSince1970
    }

    try! haptics.play()

    if (printTimings) {
        print("Play duration: ", String(format:"%f", Date().timeIntervalSince1970 - timeBefore), "s")
    }

    // Let haptics play for this many seconds.
    Thread.sleep(forTimeInterval: playDuration)

    if (stopHaptic) {
        try! haptics.stop()
    }
}

/// Prints manual test instructions to the console.
///
/// ```
/// printInstructions("testMyFeature", instructions:"Confirm that you feel...")
/// ```
///
/// - Parameters:
///   - testName: The name of the test function.
///   - instructions: The manual test instructions.
func printInstructions(testName:String, instructions:String) {
    print()
    print(testName)
    print("-------")
    print(instructions)
}
