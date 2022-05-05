//
//  MockNativeDriver.m
//  LofeltHaptics
//
//  Created by James Kneafsey on 12/05/2020.
//  Copyright © 2020 Lofelt. All rights reserved.
//

#import "MockNativeDriver.h"

@implementation MockNativeDriver

- (MockNativeDriver *_Nonnull)init {
    self = [super init];
    return self;
}

- (BOOL)handleStreamingAmplitudeEvent:(AmplitudeEvent)event error:(NSError *_Nullable *_Nullable)error {
    return YES;
}

- (BOOL)handleStreamingFrequencyEvent:(FrequencyEvent)event error:(NSError *_Nullable *_Nullable)error {
    return YES;
}

- (void)setStopCallback:(nonnull StopCallback)callback {
}

- (void)reset {
}

@end
