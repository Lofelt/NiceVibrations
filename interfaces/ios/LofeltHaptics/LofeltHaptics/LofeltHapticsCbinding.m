//
//  LofeltHapticsCbinding.m
//  LofeltHaptics
//
//  Created by Tomash Ghz on 24.11.20.
//  Copyright © 2020 Lofelt. All rights reserved.
//

// Cbinding for our iOS framework
//
// This Cbinding layer allows code that needs to speak C use our iOS framework.
// It follows a specific pattern for ownership and communication
// between the client of this code (e.g. our LofeltHaptics class in Unity)
// and the iOS framework. The pattern allows us to manage ownership and the
// lifetime of the LofeltHaptics object (./LofeltHaptics.h/m)
//
// Since we are restricted to the C language, the client code has to be
// the owner of LofeltHaptics and manage its lifetime. LofeltHaptics is written in
// Objective-C. The Objective-C compiler uses ARC (automatic reference counting).
// This means if we just:
// 1. Allocate and initialise an instance of LofeltHaptics
// 2. Try to return a pointer to the instance to the client
// Then:
// 1. The pointer will go out of scope
// 2. ARC will detect that no more references point to the instance and
// 3. Deallocate it
// 4. Client code has a dangling pointer
//
// So we need to break the reference out of ARC somehow. For this, we need another
// iOS framework that doesn't use ARC: Core Foundation. To "pass" the pointer to
// LofeltHaptics into Core Foundation we use "bridging".
//
// Bridging is a feature of MacOS/iOS development that takes account of the diverse
// frameworks that each do similar things with similar data types but are not always
// compatible so they need to be explicitly bridged. In our case, we can use bridging
// to pass ownership of the reference to LofeltHaptics to Core Foundation, thereby
// releasing it from Objective-C's ARC.
//
// CFBridgingRetain() below does exactly this and returns a CFTypeRef which is a
// wrapper for void *. As the memory for the instance of LofeltHaptics is now completely
// unmanaged, we have to explicitly release it using CFRelease(), which means the client
// code is responsible for calling lofeltHapticsReleaseBinding(). You might see
// CFBridgingRelease() in the documentation. This doesn't perform a deallocation; it just
// passes ownership back to ARC.
//
// As you see below, aside from lofeltHapticsInitBinding every function needs to get the
// pointer to LofeltHaptics and since that pointer is now in Core Foundation land it
// needs to be of type CFTypeRef.
//
// This .m file has no header. The C bindings are only used in Unity, and Unity imports
// the functions directly without using a header.

#import <LofeltHaptics/LofeltHaptics.h>
#import "SystemHaptics.h"

BOOL lofeltHapticsDeviceMeetsMinimumRequirementsBinding(void) {
    return [LofeltHaptics deviceMeetsMinimumRequirement];
}

CFTypeRef lofeltHapticsInitBinding(void) {
    NSError *error;
    if (@available(iOS 13, *)) {
        LofeltHaptics *haptics = [[LofeltHaptics alloc] initAndReturnError:&error];
        if (error != nil) {
            NSLog(@"initBinding Error: %@", error.userInfo);
            return nil;
        }
        return CFBridgingRetain(haptics);
    } else {
        return nil;
    }
    return nil;
}

// @c data is a UTF-8 encoded JSON string without the null terminator.
// The caller keeps ownership of @c data and is responsible for freeing the buffer.
BOOL lofeltHapticsLoadBinding(CFTypeRef haptics, const char* data, size_t data_size_bytes) {
    if (data == NULL) {
        NSLog(@"LofeltHaptics Error: data is null");
        return NO;
    }
    if (haptics == nil) {
        NSLog(@"LofeltHaptics Error: controller is null");
        return NO;
    }

    if (@available(iOS 13, *)) {
        NSData *nsData = [NSData dataWithBytesNoCopy:(void*)data length:data_size_bytes freeWhenDone:NO];
        NSError *error = nil;
        if ([(__bridge LofeltHaptics*)haptics loadFromData:nsData error:&error] == NO) {
            NSLog(@"LofeltHaptics Error: %@", error.userInfo);
            return NO;
        }
        return YES;
    } else {
        return NO;
    }
}

BOOL lofeltHapticsPlayBinding(CFTypeRef haptics) {
    if (@available(iOS 13, *)) {
        if (haptics != nil) {
            NSError *error;
            if ([(__bridge LofeltHaptics*)haptics play:&error] == NO) {
                NSLog(@"LofeltHaptics: %@", error.userInfo);
                return NO;
            }
        } else {
            NSLog(@"LofeltHaptics Error: controller is null");
            return NO;
        }
        return YES;
    } else {
        return NO;
    }
}

BOOL lofeltHapticsStopBinding(CFTypeRef haptics) {
    if (@available(iOS 13, *)) {
        if (haptics != nil) {
            NSError *error;
            if ([(__bridge LofeltHaptics*)haptics stop:&error] == NO) {
                NSLog(@"LofeltHaptics: %@", error.userInfo);
                return NO;
            }
        } else {
            NSLog(@"LofeltHaptics Error: controller is null");
            return NO;
        }
        return YES;
    } else {
        return NO;
    }
}

BOOL lofeltHapticsSeekBinding(CFTypeRef haptics, float time) {
    if (@available(iOS 13, *)) {
        if (haptics != nil) {
            NSError *error;
            if ([(__bridge LofeltHaptics*)haptics seek:time error:&error] == NO) {
                NSLog(@"LofeltHaptics: %@", error.userInfo);
                return NO;
            }
        } else {
            NSLog(@"LofeltHaptics Error: controller is null");
            return NO;
        }
        return YES;
    } else {
        return NO;
    }
}

BOOL lofeltHapticsSetAmplitudeMultiplicationBinding(CFTypeRef _Nonnull haptics, float factor) {
    if (@available(iOS 13, *)) {
        if (haptics == nil) {
            NSLog(@"LofeltHaptics Error: controller is null");
            return NO;
        }

        NSError *error;
        if ([(__bridge LofeltHaptics*)haptics setAmplitudeMultiplication:factor error:&error] == NO) {
            NSLog(@"LofeltHaptics: %@", error.userInfo);
            return NO;
        }

        return YES;
    }
    else {
        return NO;
    }
}

BOOL lofeltHapticsSetFrequencyShiftBinding(CFTypeRef _Nonnull haptics, float shift) {
    if (@available(iOS 13, *)) {
        if (haptics == nil) {
            NSLog(@"LofeltHaptics Error: controller is null");
            return NO;
        }

        NSError *error;
        if ([(__bridge LofeltHaptics*)haptics setFrequencyShift:shift error:&error] == NO) {
            NSLog(@"LofeltHaptics: %@", error.userInfo);
            return NO;
        }

        return YES;
    }
    else {
        return NO;
    }
}

BOOL lofeltHapticsLoopBinding(CFTypeRef haptics, BOOL enabled) {
    if (@available(iOS 13, *)) {
        if (haptics != nil) {
            NSError *error;
            if ([(__bridge LofeltHaptics*)haptics loop:enabled error:&error] == NO) {
                NSLog(@"LofeltHaptics: %@", error.userInfo);
                return NO;
            }
        } else {
            NSLog(@"LofeltHaptics Error: controller is null");
            return NO;
        }
        return YES;
    } else {
        return NO;
    }
}

float lofeltHapticsGetClipDurationBinding(CFTypeRef haptics) {
    if (@available(iOS 13, *)) {
        if (haptics != nil) {
            return [(__bridge LofeltHaptics*)haptics getClipDuration];
        } else {
            NSLog(@"LofeltHaptics Error: controller is null");
            return 0.0;
        }
    } else {
        return 0.0;
    }
}

BOOL lofeltHapticsReleaseBinding(CFTypeRef haptics) {
    if (@available(iOS 13, *)) {
        if (haptics != nil) {
            CFRelease(haptics);
        } else {
            NSLog(@"LofeltHaptics Error: controller is null");
            return NO;
        }
        return YES;
    } else {
        return NO;
    }
}

BOOL lofeltHapticsSystemHapticsInitializeBinding(void) {
    if (@available(iOS 11, *)) {
        if(SystemHapticsInitialize()) {
            return YES;
        } else {
            return NO;
        }
    } else {
        return NO;
    }
}

BOOL lofeltHapticsSystemHapticsReleaseBinding(void) {
    if (@available(iOS 11, *)) {
        SystemHapticsRelease();
        return YES;
    } else {
        return NO;
    }
}

BOOL lofeltHapticsSystemHapticsTriggerBinding(int hapticType) {
    if (@available(iOS 11, *)) {
        SystemHapticsTrigger( (SystemHapticsTypes) hapticType);
        return YES;
    } else {
        return NO;
    }
}
