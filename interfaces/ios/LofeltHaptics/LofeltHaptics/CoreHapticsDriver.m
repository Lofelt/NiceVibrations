//
//  CoreHapticsDriver.m
//  LofeltHaptics
//
//  Created by James Kneafsey on 12/05/2020.
//  Copyright © 2020 Lofelt. All rights reserved.
//

#import "CoreHapticsDriver.h"

@implementation CoreHapticsDriver

- (instancetype)initAndReturnError:(NSError *__autoreleasing  _Nullable *)error {
    self = [super init];

    _lock = [[NSLock alloc] init];
    _engineStopped = NO;

    _hapticEngine = [[CHHapticEngine alloc] initAndReturnError:error];
    if (error && *error != nil) {
        return nil;
    }

    if (![_hapticEngine startAndReturnError:error]) {
        _hapticEngine = nil;
        return nil;
    }

    // Prevent retain cycles by letting the block below only create a weak
    // reference to self.
    // See https://stackoverflow.com/a/31282939/1005419 for a more detailed discussion.
    __weak CoreHapticsDriver* weakSelf = self;

    // When the CHHapticEngine is stopped for external reasons, do two things:
    // 1. Set _engineStopped to YES, so that any further attempts by the streaming thread
    //    to play events will be ignored.
    // 2. Invoke _stopCallback. Since CHHapticEngine invokes its stoppedHandler on
    //    a dedicated Core Haptics thread, use dispatch_async() to call _stopCallback
    //    on the main thread, to avoid thread safety issues in _stopCallback.
    _hapticEngine.stoppedHandler = ^(CHHapticEngineStoppedReason reason) {
        CoreHapticsDriver* strongSelf = weakSelf;
        if (strongSelf) {
            [strongSelf->_lock lock];
            strongSelf->_engineStopped = YES;
            [strongSelf->_lock unlock];
        }

        dispatch_async(dispatch_get_main_queue(), ^{
            CoreHapticsDriver* strongSelf = weakSelf;
            if (strongSelf && strongSelf->_stopCallback) {
                strongSelf->_stopCallback();
            }
        });
    };

    _audioToHapticsPlayer = [[CoreHapticsPlayer alloc] init:_hapticEngine];
    _preauthoredHapticPlayer = [[CoreHapticsPlayer alloc] init:_hapticEngine];

    return self;
}

-(void)dealloc {
    [_lock lock];
    @try {
        // Set a stopped handler that does nothing, overriding the existing stopped
        // handler.
        // Destroying the CHHapticEngine causes the stopped handler to be invoked later,
        // from the Core Haptics thread, after CoreHapticDriver has already been destroyed.
        // Therefore the stopped handler can not access `self`, as that would access already
        // destroyed objects.
        _hapticEngine.stoppedHandler = ^(CHHapticEngineStoppedReason reason){};
    }
    @finally {
        [_lock unlock];
    }
}

- (void)reset {
    [_lock lock];
    @try {
        NSError *error = nil;
        if (![_hapticEngine startAndReturnError:&error]) {
            NSLog(@"LofeltHaptics: Restarting the Core Haptics engine failed: %@", [error localizedDescription]);
            return;
        }
        [_preauthoredHapticPlayer reset];
        [_audioToHapticsPlayer reset];
        _engineStopped = NO;
    }
    @finally {
        [_lock unlock];
    }
}

- (BOOL)handleStreamingAmplitudeEvent:(AmplitudeEvent)event error:(NSError *_Nullable *_Nullable)error {
    [_lock lock];
    @try {
        // When the engine is stopped, don't raise an error and return YES instead. It can
        // happen that this method is called while the engine is stopped, and we don't want
        // to spam the console with error messages in that case.
        if (_engineStopped) {
            return YES;
        }

        if (![_preauthoredHapticPlayer stayAwake:event.duration error:error]) {
            return NO;
        }

        if (![_preauthoredHapticPlayer playStreamingAmplitudeEvent:event error:error]) {
            return NO;
        }
    }
    @finally {
        [_lock unlock];
    }

    return YES;
}

- (BOOL)handleStreamingFrequencyEvent:(FrequencyEvent)event error:(NSError *_Nullable *_Nullable)error {
    [_lock lock];
    @try {
        // When the engine is stopped, don't raise an error and return YES instead. It can
        // happen that this method is called while the engine is stopped, and we don't want
        // to spam the console with error messages in that case.
        if (_engineStopped) {
            return YES;
        }

        if (![_preauthoredHapticPlayer stayAwake:event.duration error:error]) {
            return NO;
        }

        if (![_preauthoredHapticPlayer playStreamingFrequencyEvent:event error:error]) {
            return NO;
        }
    }
    @finally {
        [_lock unlock];
    }

    return YES;
}

- (void)setStopCallback:(StopCallback)callback
{
    _stopCallback = callback;
}

@end
