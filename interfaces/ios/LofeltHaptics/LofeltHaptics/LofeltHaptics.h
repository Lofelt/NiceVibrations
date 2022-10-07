// Copyright (c) Meta Platforms, Inc. and affiliates.
#import <Foundation/Foundation.h>
#import <AVFoundation/AVFoundation.h>

NS_ASSUME_NONNULL_BEGIN

//! Project version number for LofeltHaptics.
FOUNDATION_EXPORT double LofeltHapticsVersionNumber;

//! Project version string for LofeltHaptics.
FOUNDATION_EXPORT const unsigned char LofeltHapticsVersionString[];

//! Custom error domain
extern NSString *_Nonnull const LofeltErrorDomain;

/*!
 @class      LofeltHaptics
 @brief      The LofeltHaptics class
 @discussion Defines the API of Lofelt SDK for iOS.

             The LofeltHaptics class is not thread safe and can only be used from
             the main thread.

             When the app is put into the background, Core Haptics will not allow
             playing any haptics. LofeltHaptics will detect this situation and cease
             all activity.
             When the app is put into the foreground again, Core Haptics will allow
             playing haptics again, and LofeltHaptics re-initalizes itself. However,
             haptics that were interrupted when the app was backgrounded do not
             automatically resume and need to be started again by calling @c play().
 @author     Joao Freire, James Kneafsey, Thomas McGuire, Tomash GHz
 @copyright  © 2020 Lofelt. All rights reserved.
 */
@interface LofeltHaptics : NSObject
{
    void *_controller;
    id<NSObject> _foregroundNotificationObserver;
    id<NSObject> _backgroundNotificationObserver;
}

/*! @abstract       Checks if the iPhone meets the minimum requirements
    @discussion     This allows for a runtime check on iPhones that  won't
                    meet the requirements for Lofelt Haptics.

    @return         Whether the iPhone supports or not Lofelt Haptics
 */
+ (BOOL)deviceMeetsMinimumRequirement;

- (instancetype)init NS_UNAVAILABLE;

/*! @abstract       Creates an instance of LofeltHaptics.
    @discussion     There should only be one instance of `LofeltHaptics` created in a given application.
    @param error    If the initialization fails, this will be set to a valid NSError describing the error.
*/
- (nullable instancetype)initAndReturnError:(NSError **)error API_AVAILABLE(ios(13)) NS_SWIFT_NAME(init());

/*! @abstract       Loads a haptic clip from string data.
    @discussion     The data must be in a valid Lofelt JSON format.
                    If a haptic clip is currently playing, it will be stopped.
    @param data     The Lofelt JSON format string.
    @param error    If the load operation fails, this will be set to a valid NSError describing the error.
    @return         Whether the operation succeeded
*/
- (BOOL)load:(NSString *_Nonnull)data error:(NSError *_Nullable *_Nullable)error API_AVAILABLE(ios(13));

/*! @abstract       A version of @c load() taking @c NSData instead of @c NSString.
    @discussion     This method can be faster than @c load(), as it avoids string conversions.
    @param data     The .haptic clip, as UTF-8 encoded JSON string without a null terminator.
    @param error    If the load operation fails, this will be set to a valid NSError describing the error.
    @return         Whether the operation succeeded
*/
- (BOOL)loadFromData:(NSData *_Nonnull)data error:(NSError *_Nullable *_Nullable)error API_AVAILABLE(ios(13));

/*! @abstract       Plays a loaded haptic clip.
    @discussion     The data must be preloaded using @c load() .
                    Only one haptic clip can play at a time.
                    Playback will start from the beginning of the haptic clip, or from the seek
                    position if seek() has been called before.
                    Calling play() if the clip is already playing has no effect.
    @param error    If the play operation fails, this will be set to a valid NSError describing the error.
    @return         Whether the operation succeeded
*/
- (BOOL)play:(NSError *_Nullable *_Nullable)error API_AVAILABLE(ios(13));

/*! @abstract       Stops the haptic clip that is currently playing.
    @discussion     The call is ignored if no clip is loaded or no clip is playing.
    @param error    If the stop operation fails, this will be set to a valid NSError describing the error.
    @return         Whether the operation succeeded
 */
- (BOOL)stop:(NSError *_Nullable *_Nullable)error API_AVAILABLE(ios(13));

/*! @abstract       Jumps to a time position in the haptic clip
    @discussion     The playback state (playing or stopped) will not be changed unless seeking
                    beyond the end of the haptic clip. Seeking beyond the end of the clip will stop
                    playback.
                    Seeking to a negative position will start playback after a delay.
    @param time     The new position within the clip, as seconds from the beginning of the clip
    @param error    If the seek operation fails, this will be set to a valid NSError describing the error.
    @return         Whether the operation succeeded
 */
- (BOOL)seek:(float)time error:(NSError *_Nullable *_Nullable)error API_AVAILABLE(ios(13));

/*! @abstract                     Multiplies the amplitude of every breakpoint of the clip with the given
                                  multiplication factor
    @discussion                   In other words, this function applies a gain (for factors greater than 1.0)
                                  or an attenuation (for factors less than 1.0) to the clip.
                                  If the resulting amplitude of a breakpoint is greater than 1.0, it is
                                  clipped to 1.0. The amplitude is clipped hard, no limiter is used.
                                  The clip needs to be loaded with @c load() first. Loading a clip resets
                                  the multiplication factor back to the default of 1.0.
                                  If no clip is currently playing, the multiplication will take effect
                                  once @c play() is called. If a clip is currently playing, the multiplication
                                  will take effect immediately.
   @param amplitudeMultiplication The factor by which each amplitude will be multiplied. This value is a
                                  multiplication factor, it is not a dB value. The factor needs to be 0
                                  or greater.
   @param error                   If the operation fails, this will be set to a valid NSError describing
                                  the error. An error can for example happen if no clip is loaded, or if
                                  the factor is outside of the valid range.
   @return                        Whether the operation succeeded
 */
- (BOOL)setAmplitudeMultiplication:(float)amplitudeMultiplication error:(NSError *_Nullable *_Nullable)error API_AVAILABLE(ios(13));

/*! @abstract                     Adds the given shift to the frequency of every breakpoint in the clip,
                                  including the emphasis.
    @discussion                   In other words, this function shifts all frequencies of the clip.
                                  If the resulting frequency of a breakpoint is smaller than 0.0 or
                                  greater than 1.0, it is clipped to that range. The frequency is
                                  clipped hard, no limiter is used.
                                  The clip needs to be loaded with @c load() first. Loading a clip resets
                                  the shift back to the default of 0.0.
                                  If no clip is currently playing, the shift will take effect once @c play()
                                  is called. If a clip is currently playing, the shift will take effect
                                  immediately.
   @param shift                   The amount by which each frequency should be shifted. This number is added
                                  to each frequency value. The shift needs to be between -1.0 and 1.0.
   @param error                   If the operation fails, this will be set to a valid NSError describing
                                  the error. An error can for example happen if no clip is loaded, or if
                                  the shift is outside of the valid range.
   @return                        Whether the operation succeeded
 */
- (BOOL)setFrequencyShift:(float)shift error:(NSError *_Nullable *_Nullable)error API_AVAILABLE(ios(13));

/*! @abstract       Sets the playback to repeat from the start at the end of the clip.
    @discussion     Changes done with this function are only applied when @c play() is called.
                    When @c load() is called, looping is always disabled.
                    Playback will always start at the beginning of the clip, even if
                    @c seek() was used to jump to a different clip position before.
    @param enabled  When true, looping is set enabled; false disables looping.
    @param error    If the loop operation fails, this will be set to a valid NSError describing the error.
    @return         Whether the operation succeeded
 */
- (BOOL)loop:(BOOL)enabled error:(NSError *_Nullable *_Nullable)error API_AVAILABLE(ios(13));

/*! @abstract       Returns the duration of the loaded clip
    @discussion     It will return 0.0 for an invalid clip
    @return         Duration of the loaded clip
 */
- (float)getClipDuration;

@end

NS_ASSUME_NONNULL_END
