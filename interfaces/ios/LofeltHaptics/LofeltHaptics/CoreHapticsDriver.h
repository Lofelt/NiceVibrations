/*!
 @class         CoreHapticsDriver
 @brief         The CoreHapticsDriver class
 @discussion    This class talks directly to the CoreHaptics API. The Rust core of the
                SDK talks to CoreHapticsDriver via a C API defined in LofeltSDK.m
 @author        James Kneafsey, Tomash Ghz
 @copyright     © 2020 Lofelt. All rights reserved.
 */

#import "NativeDriver.h"
#import "CoreHapticsPlayer.h"

#import <CoreHaptics/CoreHaptics.h>
#import <Foundation/Foundation.h>

NS_ASSUME_NONNULL_BEGIN

API_AVAILABLE(ios(13.0))
@interface CoreHapticsDriver: NSObject<NativeDriver> {
    CHHapticEngine *_hapticEngine;
    CoreHapticsPlayer *_preauthoredHapticPlayer;
    StopCallback _stopCallback;

    // Some methods of this class, like handleStreamingAmplitudeEvent(), are called
    // from the streaming thread, while others, like reset(), are called from the
    // main thread. Thread safety is handled with this lock.
    NSLock* _lock;

    // Set to YES while the CHHapticEngine is stopped. While the engine is stopped,
    // CoreHapticsDriver does not attempt to play any events, which would only fail
    // and print Core Haptics error messages on the console.
    BOOL _engineStopped;
}

- (instancetype)init NS_UNAVAILABLE;

/*!
    @method     initAndReturnError:
    @abstract   Create an instance of CoreHapticsDriver.
*/
- (nullable instancetype)initAndReturnError:(NSError **)error NS_DESIGNATED_INITIALIZER;

@end

NS_ASSUME_NONNULL_END
