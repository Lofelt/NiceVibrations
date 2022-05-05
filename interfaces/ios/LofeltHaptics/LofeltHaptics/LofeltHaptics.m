//
//  LofeltHaptics.m
//  LofeltHaptics
//
//  Created by Joao Freire on 23/03/2020.
//  Copyright © 2020 Lofelt. All rights reserved.
//

#import "LofeltHaptics+ClassExtension.h"
#import "CoreHapticsDriver.h"

#import <lofelt-sdk.h>

#if !TARGET_OS_OSX
#import <UIKit/UIKit.h>
#endif
#import <Foundation/Foundation.h>
#import <os/log.h>
#import <pthread.h>
#import <mach/mach_time.h>
#import <mach/thread_act.h>

@implementation LofeltHaptics

NSString *const LofeltErrorDomain = @"com.lofelt.LofeltSDK";

+ (BOOL)deviceMeetsMinimumRequirement {
    if (@available(iOS 13, *)) {
        return CHHapticEngine.capabilitiesForHardware.supportsHaptics;
    } else {
        return NO;
    }
}

- (instancetype)initAndReturnError:(NSError *__autoreleasing _Nullable *)error {
    NSError *coreHapticsDriverError;
    if([LofeltHaptics deviceMeetsMinimumRequirement]) {
        id<NativeDriver> driver = [[CoreHapticsDriver alloc] initAndReturnError:&coreHapticsDriverError];
        if (coreHapticsDriverError != nil) {
            [self createError:@"LofeltHaptics initAndReturnError: Error initialising Core Haptics engine: " internalError:&coreHapticsDriverError externalError:error];
            return nil;
        }
        return [self initWithNativeDriverAndReturnError: driver error:error];
    } else {
        [self createError:@"LofeltHaptics initAndReturnError: This device doesn't meet the minimum requirements to play haptics. " internalError:nil externalError:error];
        return nil;
    }
}

// When suspending, we stop all playback:
// - pre-authored clip playback is stopped with a call to stop()
//
// This is only needed so that when the app comes into the foreground again,
// no code is attempting to play haptics and therefore use CoreHapticPlayer.
// Using CoreHapticPlayer after coming into the foreground would fail, as CoreHapticPlayer
// needs to be reset first.
- (void) suspend API_AVAILABLE(ios(13)) {
    NSError *error = nil;
    if (![self stop:&error]) {
        NSLog(@"LofeltHaptics: Failed to stop playback when suspending: %@", [error localizedDescription]);
    }
}

// When resuming, we:
// - Reset the CoreHapticsDriver, which will re-start the haptic engine and re-create the players
// - Re-install the audio tap we removed before in suspend()
- (void) resume API_AVAILABLE(ios(13)) {
    [_nativeDriver reset];
}

- (instancetype)initWithNativeDriverAndReturnError:(id<NativeDriver>)nativeDriver error:(NSError **)error {
    self = [super init];
    _nativeDriver = nativeDriver;

    Callbacks callbacks;
    callbacks.play_streaming_amplitude_event = &handleStreamingAmplitudeEvent;
    callbacks.play_streaming_frequency_event = &handleStreamingFrequencyEvent;
    callbacks.init_thread = &handleInitThread;
    _controller = (void *)lofelt_sdk_controller_create((__bridge void *)_nativeDriver, callbacks);
    if (_controller == nil) {
        [self getSDKError: error];
        return nil;
    }

    //
    // Suspend and resume handling is below.
    //
    // 1) When the app is about to go into the background, we suspend.
    // 2) When the app is about to go into the foreground, we resume.
    // 3) When the haptic engine is stopped for external reasons, and the app
    //   is in the foreground, we do a suspend/resume cycle.
    //   Normally, this isn't needed, as 1) and 2) handle this. However,
    //   there are cases in which Core Haptics stops the CHHapticEngine *after* the
    //   app went to the foreground, so *after* 1) and 2) have executed.
    //   This happens when the app was put into foreground very quickly after being
    //   put into the background.
    //   In this case we need to reset the engine and players again to be able to
    //   play haptics, so we do another suspend/resume cycle.
    //   This is only done if the engine stopped while being in the foreground, the
    //   usual case of the engine being stopped while in the background is handled by
    //   1) and 2).
    //

#if !TARGET_OS_OSX
    // Prevent retain cycles by letting the blocks below only create a weak
    // reference to self.
    // See https://stackoverflow.com/a/31282939/1005419 for a more detailed discussion.
    __weak LofeltHaptics* weakSelf = self;

    _backgroundNotificationObserver = [[NSNotificationCenter defaultCenter]
            addObserverForName:UIApplicationDidEnterBackgroundNotification
            object:nil
            queue:[NSOperationQueue mainQueue] usingBlock:^(NSNotification *notification) {
        if ([[notification name] isEqualToString:UIApplicationDidEnterBackgroundNotification]) {
            LofeltHaptics* strongSelf = weakSelf;
            [strongSelf suspend];
        }
    }];

    _foregroundNotificationObserver = [[NSNotificationCenter defaultCenter]
            addObserverForName:UIApplicationWillEnterForegroundNotification
            object:nil
            queue:[NSOperationQueue mainQueue] usingBlock:^(NSNotification *notification) {
        if ([[notification name] isEqualToString:UIApplicationWillEnterForegroundNotification]) {
            LofeltHaptics* strongSelf = weakSelf;
            [strongSelf resume];
        }
    }];

    [_nativeDriver setStopCallback:^{
        if ([[UIApplication sharedApplication] applicationState] != UIApplicationStateBackground) {
            LofeltHaptics* strongSelf = weakSelf;
            [strongSelf suspend];
            [strongSelf resume];
        }
    }];
#endif

    return self;
}

- (void)dealloc {
    [[NSNotificationCenter defaultCenter] removeObserver:_foregroundNotificationObserver];
    [[NSNotificationCenter defaultCenter] removeObserver:_backgroundNotificationObserver];

    if (_controller != nil) {
        lofelt_sdk_controller_destroy(_controller);
        _controller = nil;
    }
}

- (BOOL)load:(NSString *_Nonnull)data error:(NSError *_Nullable *_Nullable)error {
    NSData* nsData = [data dataUsingEncoding:NSUTF8StringEncoding];
    return [self loadFromData:nsData error:error];
}

- (BOOL)loadFromData:(NSData *_Nonnull)data error:(NSError *_Nullable *_Nullable)error {
    if (_controller == nil) {
        [self createError:@"LofeltHaptics loadFromData: Core not initialized." internalError:nil externalError:error];
        return NO;
    }

    int loadResult = lofelt_sdk_controller_load(_controller, data.bytes, data.length);
    if (loadResult == ERROR) {
        if (error != NULL) {
            [self getSDKError: error];
        }
        return NO;
    } else if (loadResult == PARTIAL_VERSION_SUPPORT) {
        os_log_info(OS_LOG_DEFAULT, "The haptic clip you are attempting to play is of a newer version "
                                    "than what is supported by this framework. Some playback features "
                                    "may not work. Please obtain the latest version of the framework "
                                    "from lofelt.com.");
    }

    return YES;
}

- (BOOL)play:(NSError *_Nullable *_Nullable)error {
    if (_controller == nil) {
        [self createError:@"LofeltHaptics play: Core not initialized." internalError:nil externalError:error];
        return NO;
    }

    if (lofelt_sdk_controller_play(_controller) == ERROR) {
        if (error != NULL) {
            [self getSDKError: error];
        }
        return NO;
    }

    return YES;
}

- (BOOL)stop:(NSError *_Nullable *_Nullable)error {
    if (_controller == nil) {
        [self createError:@"LofeltHaptics stop: Core not initialized." internalError:nil externalError:error];
        return NO;
    }

    if (lofelt_sdk_controller_stop(_controller) == ERROR){
        if (error != NULL) {
            [self getSDKError: error];
        }
        return NO;
    }

    return YES;
}

- (BOOL)seek:(float)time error:(NSError *_Nullable *_Nullable)error {
    if (_controller == nil) {
        [self createError:@"LofeltHaptics seek: Core not initialized." internalError:nil externalError:error];
        return NO;
    }

    if (lofelt_sdk_controller_seek(_controller, time) == ERROR){
        if (error != NULL) {
            [self getSDKError: error];
        }
        return NO;
    }

    return YES;
}

- (BOOL)setAmplitudeMultiplication:(float)amplitudeMultiplication error:(NSError *_Nullable *_Nullable)error API_AVAILABLE(ios(13)) {
    if (_controller == nil) {
        [self createError:@"LofeltHaptics setAmplitudeMultiplication: Core not initialized." internalError:nil externalError:error];
        return NO;
    }

    if (lofelt_sdk_controller_set_amplitude_multiplication(_controller, amplitudeMultiplication) == ERROR){
        if (error != NULL) {
            [self getSDKError: error];
        }
        return NO;
    }

    return YES;
}

- (BOOL)setFrequencyShift:(float)shift error:(NSError *_Nullable *_Nullable)error API_AVAILABLE(ios(13)) {
    if (_controller == nil) {
        [self createError:@"LofeltHaptics setFrequencyShift: Core not initialized." internalError:nil externalError:error];
        return NO;
    }

    if (lofelt_sdk_controller_set_frequency_shift(_controller, shift) == ERROR){
        if (error != NULL) {
            [self getSDKError: error];
        }
        return NO;
    }

    return YES;
}

- (BOOL)loop:(BOOL)enabled error:(NSError *_Nullable *_Nullable)error {
    if (_controller == nil) {
        [self createError:@"LofeltHaptics loop: Core not initialized." internalError:nil externalError:error];
        return NO;
    }

    if (lofelt_sdk_controller_loop(_controller, enabled) == ERROR){
        if (error != NULL) {
            [self getSDKError: error];
        }
        return NO;
    }

    return YES;
}

- (float)getClipDuration {
    if (_controller == nil) {
        return 0.0;
    }
    return lofelt_sdk_controller_get_clip_duration(_controller);
}

/*! @brief          handleStreamingAmplitudeEvent(nativeDriverPassedBack, event)
                    Callback for receiving a streaming amplitude event for pre-authored
                    clip playback from Rust.
    @discussion     The event is forwarded to the native driver, similar to how
                    handleAudioToHapticEvents() works.
                    Since the callback's signature doesn't allow returning an error,
                    any error is printed here.

    @param nativeDriverPassedBack
                    Same as in handleAudioToHapticEvents()
    @param event    The amplitude event that should be played back
*/
void handleStreamingAmplitudeEvent(void *nativeDriverPassedBack, AmplitudeEvent event) {
    if (nativeDriverPassedBack == nil) {
        NSLog(@"LofeltHaptics handleStreamingAmplitudeEvent: nativeDriverPassedBack is nil");
        return;
    }

    NSObject<NativeDriver> * nativeDriver = (__bridge NSObject<NativeDriver> *)nativeDriverPassedBack;
    NSError* error = nil;
    if (![nativeDriver handleStreamingAmplitudeEvent:event error:&error]) {
        NSLog(@"LofeltHaptics: Could not play amplitude event: %@", [error localizedDescription]);
    }
}

/*! @brief          handleStreamingFrequencyEvent(nativeDriverPassedBack, event)
                    Callback for receiving a streaming frequency event for pre-authored
                    clip playback from Rust.
    @discussion     Same as handleStreamingAmplitudeEvent(), but for frequency instead of
                    amplitude events
    @param nativeDriverPassedBack
                    Same as in handleAudioToHapticEvents()
    @param event    The frequency event that should be played back
*/
void handleStreamingFrequencyEvent(void *nativeDriverPassedBack, FrequencyEvent event) {
    if (nativeDriverPassedBack == nil) {
        NSLog(@"LofeltHaptics handleStreamingFrequencyEvent: nativeDriverPassedBack is nil");
        return;
    }

    NSObject<NativeDriver> * nativeDriver = (__bridge NSObject<NativeDriver> *)nativeDriverPassedBack;
    NSError * error = nil;
    if (![nativeDriver handleStreamingFrequencyEvent:event error:&error]) {
        NSLog(@"LofeltHaptics: Could not play frequency event: %@", [error localizedDescription]);
    }
}

/*! @brief          handleInitThread()
                    Callback for initializing the streaming thread, in particular for
                    increasing the thread priority
*/
void handleInitThread(void) {
    // Move the thread to the realtime priority band using thread_policy_set(). This will cause the
    // kernel to run our thread with much less wakeup latency after a timeout compared to the normal
    // priority band. This ensures each breakpoint is played close to the designed time.
    //
    // We need to give thread_policy_set() some numbers on how long the computations
    // in this thread take, and how often they happen. The kernel will then do the best
    // to schdedule this thread to meet these requirements. These numbers need to be
    // reasonably accurate, as the kernel will demote the thread if we lie blatently about
    // the requirements. The numbers themselves don't seem to have any influence on the jitter.
    //
    // Therefore I measured how long the commands in the thread's main loop took to execute,
    // on an iPhone SE (2nd generation), in release mode, playing the "Achievement" clip:
    // Playing an event (after timeout): ~500µs
    // Loading a clip: ~1000ns
    // Playing a clip: ~1000ns
    // Lofelt Studio's maximum setting for converting audio to haptics is 60 breakpoints per
    // second, which is ~17ms per breakpoint. Manual editing allows for breakpoints to be less
    // than 1ms apart though.
    //
    // We could say that every 17 milliseconds, we need 500µs of computation time. That would be
    // a CPU load of ~3%. To be a bit on the safe side, we instead say that every 10ms, we need
    // 1ms of computation time, which is a CPU load of 10%.
    //
    // See https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/KernelProgramming/scheduler/scheduler.html
    // for more information.
    mach_timebase_info_data_t timebase;
    if (mach_timebase_info(&timebase) != KERN_SUCCESS) {
        NSLog(@"LofeltHaptics: Unable to set realtime policy for the streaming thread, unable to query timing.");
        return;
    }
    double milliseconds_to_absolute_time = timebase.denom / (double)timebase.numer * 1000.0 * 1000.0;
    struct thread_time_constraint_policy policy;
    policy.period = 10 * milliseconds_to_absolute_time; // One event every 10ms
    policy.computation = milliseconds_to_absolute_time; // One event takes 1ms of computation time
    policy.constraint = 2.5 * milliseconds_to_absolute_time; // Max limit on how long our computation should take, 2.5ms
    policy.preemptible = FALSE; // We don't want to get interrupted while playing an event
    thread_port_t port = pthread_mach_thread_np(pthread_self());
    if (thread_policy_set(port, THREAD_TIME_CONSTRAINT_POLICY, (thread_policy_t)&policy, THREAD_TIME_CONSTRAINT_POLICY_COUNT) != KERN_SUCCESS) {
        NSLog(@"LofeltHaptics: Unable to set realtime policy for the streaming thread.");
        return;
    }
}

- (void)createError:(NSString *)message internalError:(NSError *_Nullable *_Nullable)internalError externalError:(NSError *_Nullable *_Nullable)externalError {
    if (externalError == NULL) {
        return;
    }

    if (internalError != NULL && *internalError != NULL && (*internalError).userInfo != NULL) {
        message = [message stringByAppendingString:(*internalError).localizedDescription];
    }

    // Construct NSError.
    NSDictionary *errDict = @{NSLocalizedDescriptionKey: message};
    *externalError = [[NSError alloc] initWithDomain: LofeltErrorDomain code:1312 userInfo:errDict];
}

- (void)getSDKError: (NSError *_Nullable *_Nullable)error {
    if (error == NULL) {
        return;
    }

    // get the error string from SDK
    int bufferSize = lofelt_sdk_get_error_message_length();
    NSMutableData *bytes = [NSMutableData dataWithLength:sizeof(char) * bufferSize];
    char *buffer = (char *)[bytes mutableBytes];

    if (lofelt_sdk_get_error_message(buffer, bufferSize) != ERROR) { // successfully retrieved error message
        // Construct NSError
        NSString *description = [NSString stringWithUTF8String:buffer];
        NSDictionary *errDict = @{NSLocalizedDescriptionKey: description};
        *error = [[NSError alloc] initWithDomain: LofeltErrorDomain code:1312 userInfo:errDict];
    }
}

@end
