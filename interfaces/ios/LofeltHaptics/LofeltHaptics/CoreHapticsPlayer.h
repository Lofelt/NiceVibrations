/*!
@class         CoreHapticsPlayer
@brief         The CoreHapticsPlayer class
@discussion    This class manages a CHHapticPatternPlayer and creates a new one before it goes asleep
               (which happens every 30 seconds).
@copyright     Meta Platforms, Inc. and affiliates. Confidential and proprietary.
*/

#import "NativeDriver.h"
#import "HapticRamp.h"

#import <CoreHaptics/CoreHaptics.h>
#import <Foundation/Foundation.h>

NS_ASSUME_NONNULL_BEGIN

API_AVAILABLE(ios(13.0))
@interface CoreHapticsPlayer : NSObject {
    CHHapticEngine *_hapticEngine;
    id<CHHapticPatternPlayer> _player;

    // The CHHapticPatternPlayer above stops running after 30 seconds. We manage
    // to play past this limit by creating a new CHHapticPatternPlayer after 29
    // seconds. We transfer the currently playing intensity and sharpness
    // parameter curve to this new CHHapticPatternPlayer.

    // When we start a new player we calculate the time at which it will sleep
    // as being 29 seconds from now. Then we use _sleepTime to decide
    // when to create a new CHHapticPatternPlayer.
    NSDate *_sleepTime;

    // The intensity and sharpness events that are currently being played out
    // on _player.
    HapticRamp *_intensity;
    HapticRamp *_sharpness;
}

/*!
    @method     init:
    @abstract   Create an instance of CoreHapticsPlayer.
*/
- (nullable instancetype)init:(CHHapticEngine *)hapticEngine;

/*!
    @abstract                   Keeps playback running by keeping CHHapticPatternPlayers awake.
    @discussion                 Achieves this by checking if the self._player will go asleep before
                                secondsToStayAwake. If so start() is called to create a new
                                CHHapticPatternPlayer.
    @param secondsToStayAwake   The number of seconds _player has to stay awake.
*/
- (BOOL)stayAwake:(double)secondsToStayAwake error:(NSError *_Nullable *_Nullable)error;

/*
 Creates and plays a parameter curve based on the passed event.

 We always ramp (interpolate) from the current amplitude to the amplitude of the
 passed event.
*/
- (BOOL)playStreamingAmplitudeEvent:(AmplitudeEvent)event error:(NSError *_Nullable *_Nullable)error;

/*
 Same as playStreamingAmplitudeEvent(), but for a frequency event
*/
- (BOOL)playStreamingFrequencyEvent:(FrequencyEvent)event error:(NSError *_Nullable *_Nullable)error;

/*
 Resets CoreHapticsPlayer so that when the next event is played, it will be played
 on a new CHHapticPatternPlayer with a fresh HapticRamp.
 */
- (void)reset;

@end

NS_ASSUME_NONNULL_END
