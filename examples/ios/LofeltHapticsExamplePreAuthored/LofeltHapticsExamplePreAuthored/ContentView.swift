
// (c) Meta Platforms, Inc. and affiliates. Confidential and proprietary.

import SwiftUI
import LofeltHaptics
import AVFoundation

var audioPlayer: AVAudioPlayer?
var haptics: LofeltHaptics?

struct ContentView: View {
    var hapticsSupported = false;
    
    init() {
        // instantiate haptics player
        haptics = try? LofeltHaptics.init()
        
        // check if device supports Lofelt Haptics
        hapticsSupported = LofeltHaptics.deviceMeetsMinimumRequirement()
    }
    
    var body: some View {
        if hapticsSupported {
            VStack {
                HStack {
                    Button(action: {
                        // load audio clip
                        let audioData = NSDataAsset(name: "Achievement_1-audio")
                        audioPlayer = try? AVAudioPlayer(data: audioData!.data)
                        
                        // load haptic clip
                        try? haptics?.load(from: self.loadHapticData(fileName: "Achievement_1.haptic"))
                        
                        // play audio and haptic clip
                        audioPlayer?.play()
                        try? haptics?.play()
                    }) {
                        HStack {
                            Text("Achievement")
                        }
                    }
                    .shadow(radius: 20.0)
                    .padding()
                    .background(Color.gray)
                    .cornerRadius(10.0)
                    
                    Button(action: {
                        // load audio clip
                        let audioData = NSDataAsset(name: "Stroke_1-audio")
                        audioPlayer = try? AVAudioPlayer(data: audioData!.data)
                        
                        // load haptic clip
                        try? haptics?.load(from: self.loadHapticData(fileName: "Stroke_1.haptic"))
                        
                        // play audio and haptic clip
                        audioPlayer?.play()
                        try? haptics?.play()
                    }) {
                        HStack {
                            Text("Stroke")
                        }
                    }
                    .shadow(radius: 20.0)
                    .padding()
                    .background(Color.gray)
                    .cornerRadius(10.0)
                    
                }.padding()
                
            }
            .foregroundColor(Color.white)
            .font(.subheadline)
        } else {
            Text("Lofelt Haptics is not supported on this device. \r\nMinimum requirements: at least iOS 13 and iPhone 8 or newer.")
        }
    }
    
    func loadHapticData(fileName: String) -> Data {
        let hapticData = NSDataAsset(name: fileName)
        return hapticData!.data
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
