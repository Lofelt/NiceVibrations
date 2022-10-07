// Copyright (c) Meta Platforms, Inc. and affiliates.

#import "CoreHapticsPlayer.h"

@implementation CoreHapticsPlayer

const float AMPLITUDE_DUCKING = 0.2;

// CHPatternPlayer stops running after 30 seconds; we restart after 29 seconds.
const float TIME_LIMIT = 29.0;

- (nullable instancetype)init:(CHHapticEngine *)hapticEngine {
    self = [super init];

    _hapticEngine = hapticEngine;

    _intensity = [[HapticRamp alloc] init];
    _sharpness = [[HapticRamp alloc] init];

    _sleepTime = [NSDate date];
    _player = nil;

    return self;
}

- (void)reset {
    // Reset the ramps, so that when the next event is played, no leftover from
    // the previous event is included.
    _intensity = [[HapticRamp alloc] init];
    _sharpness = [[HapticRamp alloc] init];

    // Set the _player to nil here, to destroy it. A new one will be created
    // the next time stayAwake() is called.
    //
    // It would also be possible to also call start() here already, but that isn't
    // needed yet, and can wait until the next event is played.
    _player = nil;
}

- (BOOL)stayAwake:(double)secondsToStayAwake error:(NSError * _Nullable __autoreleasing *)error {
    // If end time of the incoming event > 29s of the CHHapticPatternPlayer's awake time
    // Create new CHHapticPatternPlayer now
    // Set the output of the existing one to zero
    // We want to stay awake till: now + secondsToStayAwake
    NSDate *timeStayAwakeTill = [[NSDate date] dateByAddingTimeInterval:secondsToStayAwake];
    if (_player == nil || [_sleepTime compare:timeStayAwakeTill] == NSOrderedAscending) {
        return [self start:error];
    }

    return YES;
}

/*!
    @abstract   Starts a new CHHapticPatternPlayer and transfers any currently playing haptic events
                to it.
    @discussion Creates a simple CHHapticPattern with intensity of 1.0 and a sharpness of 0.0.
                This pattern is played and then modulated by incoming haptic events with parameter
                curves. Currently playing haptic events are transferred to the new CHHapticPatternPlayer
                by splitting them at the current time and moving the remainder of the event to
                the new player.
*/
- (BOOL)start:(NSError * _Nullable __autoreleasing *)error {
    if (_hapticEngine == nil) {
        return NO;
    }

    //
    // Step 1: Create a new CHHapticPatternPlayer and assign it to _player.
    //         If any operation here fails, we return early and leave all member
    //         variables untouched.
    //

    // Create a haptic pattern that simply plays a continuous event with an intensity of 1.0 and a sharpness of 0.0.
    // This will be modulated in the methods below.
    CHHapticEventParameter *intensity = [[CHHapticEventParameter alloc]
                                         initWithParameterID:CHHapticEventParameterIDHapticIntensity
                                                    value:1.0];
    CHHapticEventParameter *sharpness = [[CHHapticEventParameter alloc]
                                         initWithParameterID:CHHapticEventParameterIDHapticSharpness
                                                       value:0.0];

    CHHapticEvent *hapticEvent = [[CHHapticEvent alloc] initWithEventType:CHHapticEventTypeHapticContinuous
                                                               parameters:@[intensity, sharpness]
                                                             relativeTime:0
                                                                 duration:30.0];

    CHHapticPattern *pattern = [[CHHapticPattern alloc] initWithEvents:@[hapticEvent] parameters:@[] error:error];
    if (error && *error != nil) {
        return NO;
    }

    // Create the haptic pattern player from this pattern.
    id<CHHapticPatternPlayer> newPlayer = [_hapticEngine createPlayerWithPattern:pattern error:error];
    if (error && *error != nil) {
        return NO;
    }

    // Get the pattern player started up and ready to be modulated.
    if (![newPlayer startAtTime:0 error:error]) {
        return NO;
    }

    // Stop the old player.
    if (_player != nil) {
        if (![_player stopAtTime:0 error:error]) {
            return NO;
        }
    }

    _player = newPlayer;

    //
    // Step 2: Create control points based on the _intensity and _sharpness ramps,
    //         and play them with playParameterCurve().
    //

    // Split current events since the player will be re-started.
    [_intensity split];
    [_sharpness split];

    // Create control points to be played with values coming from the split ramp.
    NSMutableArray<CHHapticParameterCurveControlPoint *> *controlPointsIntensity = [NSMutableArray arrayWithCapacity:2];
    NSMutableArray<CHHapticParameterCurveControlPoint *> *controlPointsSharpness = [NSMutableArray arrayWithCapacity:2];

    CHHapticParameterCurveControlPoint *controlPointIntensityStart  = [[CHHapticParameterCurveControlPoint alloc]
                                                                        initWithRelativeTime:0.0
                                                                        value:_intensity.start_value];

    CHHapticParameterCurveControlPoint *controlPointIntensityEnd    = [[CHHapticParameterCurveControlPoint alloc]
                                                                        initWithRelativeTime:[_intensity getDuration]
                                                                        value:_intensity.end_value];

    CHHapticParameterCurveControlPoint *controlPointSharpnessStart  = [[CHHapticParameterCurveControlPoint alloc]
                                                                        initWithRelativeTime:0.0
                                                                        value:_sharpness.start_value];

    CHHapticParameterCurveControlPoint *controlPointSharpnessEnd    = [[CHHapticParameterCurveControlPoint alloc]
                                                                        initWithRelativeTime:[_sharpness getDuration]
                                                                        value:_sharpness.end_value];

    [controlPointsIntensity addObject:controlPointIntensityStart];
    [controlPointsIntensity addObject:controlPointIntensityEnd];

    [controlPointsSharpness addObject:controlPointSharpnessStart];
    [controlPointsSharpness addObject:controlPointSharpnessEnd];

    _sleepTime = [[NSDate date] dateByAddingTimeInterval:TIME_LIMIT];

    // Play split ramps.
    if (![self playParameterCurve:controlPointsIntensity
                      parameterID:CHHapticDynamicParameterIDHapticIntensityControl
                            error:error]) {
        return NO;
    }

    return [self playParameterCurve:controlPointsSharpness
                        parameterID:CHHapticDynamicParameterIDHapticSharpnessControl
                              error:error];
}


- (BOOL)playStreamingAmplitudeEvent:(AmplitudeEvent)event error:(NSError *_Nullable *_Nullable)error {
    CHHapticParameterCurveControlPoint* controlPointCurrent = [[CHHapticParameterCurveControlPoint alloc]
                                                               initWithRelativeTime:0.0
                                                                              value:_intensity.end_value];

    // reason for sqrt() used in the calls below can be found here in pages from 8 to 10
    // https://docs.google.com/presentation/d/1XX1lm4WLANF1wXDk0TnkBcPSx4QfneYybNDmksS5I2Q/edit?usp=drivesdk

    // Event with emphasis
    if (!isnan(event.emphasis.amplitude) && !isnan(event.emphasis.frequency)) {
        [_intensity chainNextValue:event.duration end_value:sqrt(event.amplitude) * (1.0 - AMPLITUDE_DUCKING)];
        if (![self playTransient:sqrt(event.emphasis.amplitude)
                       sharpness:event.emphasis.frequency
                           error:error]) {
            return NO;
        }
    }

    // Event without emphasis
    else {
        [_intensity chainNextValue:event.duration end_value:sqrt(event.amplitude)];
    }

    CHHapticParameterCurveControlPoint *controlPointNext = [[CHHapticParameterCurveControlPoint alloc]
                                                            initWithRelativeTime:[_intensity getDuration]
                                                                           value:_intensity.end_value];

    NSMutableArray<CHHapticParameterCurveControlPoint *> *controlPoints = [NSMutableArray arrayWithCapacity:2];
    [controlPoints addObject:controlPointCurrent];
    [controlPoints addObject:controlPointNext];

    return [self playParameterCurve:controlPoints parameterID:CHHapticDynamicParameterIDHapticIntensityControl
                              error:error];
}

- (BOOL)playStreamingFrequencyEvent:(FrequencyEvent)event error:(NSError *_Nullable *_Nullable)error {
    CHHapticParameterCurveControlPoint* controlPointCurrent = [[CHHapticParameterCurveControlPoint alloc]
                                                               initWithRelativeTime:0.0
                                                                              value:_sharpness.end_value];
    [_sharpness chainNextValue:event.duration end_value:sqrt(event.frequency)];
    CHHapticParameterCurveControlPoint *controlPointNext = [[CHHapticParameterCurveControlPoint alloc]
                                                            initWithRelativeTime:[_sharpness getDuration]
                                                                           value:_sharpness.end_value];

    NSMutableArray<CHHapticParameterCurveControlPoint *> *controlPoints = [NSMutableArray arrayWithCapacity:2];
    [controlPoints addObject:controlPointCurrent];
    [controlPoints addObject:controlPointNext];

    return [self playParameterCurve:controlPoints
                        parameterID:CHHapticDynamicParameterIDHapticSharpnessControl
                              error:error];
}

- (BOOL)playParameterCurve:(NSMutableArray<CHHapticParameterCurveControlPoint *> *)controlPoints
               parameterID:(CHHapticDynamicParameterID)parameterID
                     error:(NSError *_Nullable *_Nullable)error {
    if (controlPoints.count > 0) {
        CHHapticParameterCurve *parameterCurve = [[CHHapticParameterCurve alloc] initWithParameterID:parameterID
                                                                                       controlPoints:controlPoints
                                                                                        relativeTime:0.0];

        // Schedule the parameter curves on the haptic pattern player.
        // If there is an error scheduling curves, we stop playback immediately so that the user
        // is left with nothing, rather than unwanted haptics.
        if (![_player scheduleParameterCurve:parameterCurve atTime:0.0 error:error]) {
            NSLog(@"LofeltHaptics: Cannot schedule parameter curve. Stopping playback completely");

            if (![_player stopAtTime:0 error:error]) {
                NSLog(@"LofeltHaptics: Unable to stop playback");
            }

            return NO;
        }
    }

    return YES;
}

- (BOOL)playTransient:(float)intensity
            sharpness:(float)sharpness
                error:(NSError *_Nullable *_Nullable)error
{
    if (!_hapticEngine) {
        return NO;
    }

    CHHapticEventParameter *intensityParameter = [[CHHapticEventParameter alloc]
                                                  initWithParameterID:CHHapticEventParameterIDHapticIntensity
                                                                value:intensity];
    CHHapticEventParameter *sharpnessParameter = [[CHHapticEventParameter alloc]
                                                  initWithParameterID:CHHapticEventParameterIDHapticSharpness
                                                                value:sharpness];

    CHHapticEvent *event = [[CHHapticEvent alloc] initWithEventType:CHHapticEventTypeHapticTransient
                                                         parameters:@[intensityParameter, sharpnessParameter]
                                                       relativeTime:0];

    CHHapticPattern *pattern = [[CHHapticPattern alloc] initWithEvents:@[event]
                                                            parameters:@[]
                                                                 error:error];

    // Exit early.
    if (error && *error != nil) {
        return NO;
    }

    id<CHHapticPatternPlayer> hapticPatternPlayer = [_hapticEngine createPlayerWithPattern:pattern
                                                                                     error:error];

    // Exit early.
    if (error && *error != nil) {
        return NO;
    }

    if (![hapticPatternPlayer startAtTime:0 error:error]) {
        return NO;
    }

    return YES;
}

@end

