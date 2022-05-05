//
//  SystemHaptics.h
//  LofeltHaptics
//
//  Created by Joao Freire on 23/08/2021.
//  Copyright © 2021 Lofelt. All rights reserved.
//
#import <Foundation/Foundation.h>
#import <AVFoundation/AVFoundation.h>

// UIKit is not available for macOS so this check is needed
// since we have tests running on macOS
#if !TARGET_OS_OSX
#import <UIKit/UIKit.h>

/*! @abstract                    Represents the all the different types of system haptics that are possible to be triggered
 */
typedef NS_ENUM(NSUInteger, SystemHapticsTypes) {
    Selection       = 0,
    Success         = 1,
    Warning         = 2,
    Failure         = 3,
    LightImpact     = 4,
    MediumImpact    = 5,
    HeavyImpact     = 6,
    RigidImpact     = 7,
    SoftImpact      = 8,
    None            = -1
};

/*! @abstract                     Initializes the @c UIFeedbackGenerators to be triggered by @c SystemHapticsTrigger()
    @discussion                   It will initialize a @c UISelectionFeebackGenerator, a  @c UINotificationFeedbackGenerator
                                  and multiple @c UIImpactFeedbackGenerator.
    @return                       Returns false in case there wasn't possible to initialize all @c UIFeedbackGenerators associated.
 */
BOOL SystemHapticsInitialize(void);

/*! @abstract                     Triggers predefined iOS system haptics provided by @c UIFeedbackGenerator
    @discussion                   Simplifies triggering system haptics provided by the @c UIFeedbackGenerator API.
    @param hapticType             Indicates which type of system haptics to be played, represented by @c SystemHapticsTypes.
 */
void SystemHapticsTrigger(SystemHapticsTypes hapticType);

/*! @abstract                     Releases the @c UIFeedbackGenerators initialized by @c SystemHapticsInitialize()
 */
void SystemHapticsRelease(void);

#endif
