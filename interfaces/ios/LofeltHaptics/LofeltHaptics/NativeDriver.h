// Copyright (c) Meta Platforms, Inc. and affiliates.
/*!
 @protocol      NativeDriver
 
 @brief         NativeDriver protocol
 
 @discussion    A protocol to allow LofeltHaptics to work with different types of native drivers.
 @author        James Kneafsey
 @copyright     © 2020 Lofelt. All rights reserved.
 @version
 */

#import <Foundation/Foundation.h>
#import <lofelt-sdk.h>

NS_ASSUME_NONNULL_BEGIN

typedef void (^StopCallback)(void);

@protocol NativeDriver

/*!
    @method             handleStreamingAmplitudeEvent:event:error
    @abstract           Handles a single amplitude streaming event.
    @param event        The event.
*/
- (BOOL)handleStreamingAmplitudeEvent:(AmplitudeEvent)event error:(NSError *_Nullable *_Nullable)error;

/*!
    @method             handleStreamingFrequencyEvent:event:error
    @abstract           Handles a single frequency streaming event.
    @param event        The event.
*/
- (BOOL)handleStreamingFrequencyEvent:(FrequencyEvent)event error:(NSError *_Nullable *_Nullable)error;

/*!
    @method             setStopCallback:callback
    @abstract           Sets a callback that is invoked on the main thread when the playback
                        is stopped for external reasons, for example when the app is suspended
                        into the background.
    @param callback     The callback.
*/
- (void)setStopCallback:(StopCallback)callback;

/*!
     @method             reset
     @abstract           Resets the driver, by re-starting and re-creating all
                         objects, such as the engine and the players.
 */
- (void)reset;

@end

NS_ASSUME_NONNULL_END
