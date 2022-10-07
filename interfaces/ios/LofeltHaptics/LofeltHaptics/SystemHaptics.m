// Copyright (c) Meta Platforms, Inc. and affiliates.

#import "SystemHaptics.h"

UISelectionFeedbackGenerator* selectionFeedbackGenerator = nil;
UINotificationFeedbackGenerator* notificationFeedbackGenerator = nil;
UIImpactFeedbackGenerator* lightImpactFeedbackGenerator = nil;
UIImpactFeedbackGenerator* mediumImpactFeedbackGenerator = nil;
UIImpactFeedbackGenerator* heavyImpactFeedbackGenerator = nil;
UIImpactFeedbackGenerator* rigidImpactFeedbackGenerator = nil;
UIImpactFeedbackGenerator* softImpactFeedbackGenerator = nil;

BOOL SystemHapticsInitialize()
{
    selectionFeedbackGenerator = [[UISelectionFeedbackGenerator alloc] init];
    notificationFeedbackGenerator = [[UINotificationFeedbackGenerator alloc] init];
    lightImpactFeedbackGenerator = [[UIImpactFeedbackGenerator alloc] initWithStyle:UIImpactFeedbackStyleLight];
    mediumImpactFeedbackGenerator = [[UIImpactFeedbackGenerator alloc] initWithStyle:UIImpactFeedbackStyleMedium];
    heavyImpactFeedbackGenerator = [[UIImpactFeedbackGenerator alloc] initWithStyle: UIImpactFeedbackStyleHeavy];
    if (@available(iOS 13, *))
    {
      rigidImpactFeedbackGenerator = [[UIImpactFeedbackGenerator alloc] initWithStyle: UIImpactFeedbackStyleRigid];
      softImpactFeedbackGenerator = [[UIImpactFeedbackGenerator alloc] initWithStyle: UIImpactFeedbackStyleSoft];
    }
    else
    {
      rigidImpactFeedbackGenerator = [[UIImpactFeedbackGenerator alloc] initWithStyle: UIImpactFeedbackStyleHeavy];
      softImpactFeedbackGenerator = [[UIImpactFeedbackGenerator alloc] initWithStyle: UIImpactFeedbackStyleLight];
    }
    
    if( !selectionFeedbackGenerator    ||
        !notificationFeedbackGenerator ||
        !lightImpactFeedbackGenerator  ||
        !mediumImpactFeedbackGenerator ||
        !heavyImpactFeedbackGenerator  ||
        !rigidImpactFeedbackGenerator  ||
        !softImpactFeedbackGenerator) {
        return NO;
    } else {
        return YES;
    }
}

void SystemHapticsRelease()
{
    selectionFeedbackGenerator = nil;
    notificationFeedbackGenerator = nil;
    lightImpactFeedbackGenerator = nil;
    mediumImpactFeedbackGenerator = nil;
    heavyImpactFeedbackGenerator = nil;
    rigidImpactFeedbackGenerator = nil;
    softImpactFeedbackGenerator = nil;
}

void triggerSelectionFeedbackGenerator(void)
{
    if(selectionFeedbackGenerator != nil) {
        [selectionFeedbackGenerator prepare];
        [selectionFeedbackGenerator selectionChanged];
    }
}

void triggerSuccessFeedbackGenerator(void)
{
    if(notificationFeedbackGenerator != nil) {
        [notificationFeedbackGenerator prepare];
        [notificationFeedbackGenerator notificationOccurred:UINotificationFeedbackTypeSuccess];
    }
}

void triggerWarningFeedbackGenerator(void)
{
    if(notificationFeedbackGenerator != nil) {
        [notificationFeedbackGenerator prepare];
        [notificationFeedbackGenerator notificationOccurred:UINotificationFeedbackTypeWarning];
    }
}

void triggerFailureFeedbackGenerator(void)
{
    if(notificationFeedbackGenerator != nil) {
        [notificationFeedbackGenerator prepare];
        [notificationFeedbackGenerator notificationOccurred:UINotificationFeedbackTypeError];
    }
}

void triggerLightImpactFeedbackGenerator(void)
{
    if(lightImpactFeedbackGenerator != nil) {
        [lightImpactFeedbackGenerator prepare];
        [lightImpactFeedbackGenerator impactOccurred];
    }
}

void triggerMediumImpactFeedbackGenerator(void)
{
    if(mediumImpactFeedbackGenerator != nil) {
        [mediumImpactFeedbackGenerator prepare];
        [mediumImpactFeedbackGenerator impactOccurred];
    }
}

void triggerHeavyImpactFeedbackGenerator(void)
{
    if(heavyImpactFeedbackGenerator != nil) {
        [heavyImpactFeedbackGenerator prepare];
        [heavyImpactFeedbackGenerator impactOccurred];
    }
}

void triggerRigidImpactFeedbackGenerator(void)
{
    if(rigidImpactFeedbackGenerator != nil) {
        [rigidImpactFeedbackGenerator prepare];
        [rigidImpactFeedbackGenerator impactOccurred];
    }
}

void triggerSoftImpactFeedbackGenerator(void)
{
    if(softImpactFeedbackGenerator != nil) {
        [softImpactFeedbackGenerator prepare];
        [softImpactFeedbackGenerator impactOccurred];
    }
}

void SystemHapticsTrigger(SystemHapticsTypes hapticType) {
    switch(hapticType) {
        case Selection:
            triggerSelectionFeedbackGenerator();
            break;
        case Success:
            triggerSuccessFeedbackGenerator();
            break;
        case Warning:
            triggerWarningFeedbackGenerator();
            break;
        case Failure:
            triggerFailureFeedbackGenerator();
            break;
        case LightImpact:
            triggerLightImpactFeedbackGenerator();
            break;
        case MediumImpact:
            triggerMediumImpactFeedbackGenerator();
            break;
        case HeavyImpact:
            triggerHeavyImpactFeedbackGenerator();
            break;
        case RigidImpact:
            triggerRigidImpactFeedbackGenerator();
            break;
        case SoftImpact:
            triggerSoftImpactFeedbackGenerator();
            break;
        default:
            //Nothing to do
            break;
    }
}


