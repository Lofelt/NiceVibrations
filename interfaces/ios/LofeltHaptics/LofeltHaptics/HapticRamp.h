// Copyright (c) Meta Platforms, Inc. and affiliates.
/*!
@class         HapticRamp
@brief         The HapticRamp class
@discussion    A ramp between two breakpoints. This is used to chain breakpoints together for streamed haptic playback.
@copyright     Meta Platforms, Inc. and affiliates. Confidential and proprietary. 
*/

#import <Foundation/Foundation.h>

NS_ASSUME_NONNULL_BEGIN

@interface HapticRamp : NSObject {
}

@property NSDate *start_time;           // The time the ramp actually started.
@property NSDate *end_time;             // The time the ramp should end.
@property float start_value;            // The value at which the ramp started.
@property float end_value;              // The value the ramp should have at `end_time`

/*!
    @method     init:
    @abstract   Create an instance of HapticRamp.
*/
- (nullable instancetype)init;

/*!
    @method     chainNextValue:duration:end_value
    @abstract   Sets the event to start now, for the given duration, with the start value the event
                currently has and the end value passed in.
*/
- (void)chainNextValue:(NSTimeInterval)duration end_value:(double)end_value;

/*!
    @method     getDuration
    @abstract   Returns the duration in seconds.
*/
- (NSTimeInterval)getDuration;

/*!
    @method     split
    @abstract   Splits the ramp at whatever time and value it has now and considers that to be
                its start time and start value.
                This is needed for when a new CHHapticPatternPlayer is started and this ramp
                needs to be transferred to it and continue from where it was.
*/
- (void)split;

@end

NS_ASSUME_NONNULL_END
