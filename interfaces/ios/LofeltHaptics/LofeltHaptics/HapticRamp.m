// Copyright (c) Meta Platforms, Inc. and affiliates.

#import "HapticRamp.h"

@implementation HapticRamp

- (nullable instancetype)init
{
    self = [super init];

    self.start_time = [NSDate date];
    self.end_time = [NSDate date];
    self.start_value = 0;
    self.end_value = 0;

    return self;
}

// Sets the start time to now and the end time to now + duration.
// Sets the start value to the end value of the previous ramp.
// Sets the end value to the value given.
- (void)chainNextValue:(NSTimeInterval)duration end_value:(double)end_value
{
    self.start_time = [NSDate date];
    self.end_time = [self.start_time dateByAddingTimeInterval:duration];
    self.start_value = self.end_value;
    self.end_value = end_value;
}

- (NSTimeInterval)getDuration
{
    NSTimeInterval duration = [self.end_time timeIntervalSinceDate:self.start_time];

    // CoreHaptics doesn't respond well to ramps with a duration of 0,
    // the ramp either does not play, or it produces glitches (see PD-3106).
    const double minimumDuration = 0.001;

    return MAX(duration, minimumDuration);
}

// Sets the start time to now; leaves the end time as it is.
// Sets the start value to whatever the interpolated value is (i.e. what
// should be playing right now on the CHHapticPatternPlayer).
// Leaves the end value as it is.
- (void)split
{
    NSTimeInterval from_start_till_now = [[NSDate date] timeIntervalSinceDate:self.start_time];
    double lerpPosition = 0.0;
    double duration = [self getDuration];

    //check if duration is 0.0 to avoid divide by zero NAN value.
    if (duration != 0.0) {
        lerpPosition = from_start_till_now / [self getDuration];
    }

    self.start_value = self.start_value + (self.end_value - self.start_value) * lerpPosition;

    self.start_time = [NSDate date];
}

@end
