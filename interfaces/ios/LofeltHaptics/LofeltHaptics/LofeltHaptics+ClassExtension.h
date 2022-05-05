//
//  LofeltHaptics+ClassExtension.h
//  LofeltHaptics
//
//  Created by James Kneafsey on 28/05/2020.
//  Copyright © 2020 Lofelt. All rights reserved.
//

/*!
@header     LofeltHaptics+ClassExtension.h
@brief      This is a class extension to LofeltHaptics.
@discussion Class extensions provide a mechanism to declare private instance variables
            that should not be shipped with the public header.
            https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ProgrammingWithObjectiveC/CustomizingExistingClasses/CustomizingExistingClasses.html#//apple_ref/doc/uid/TP40011210-CH6-SW3
@author     James Kneafsey
@copyright  © 2020 Lofelt. All rights reserved.
*/
#import "LofeltHaptics.h"
#import "NativeDriver.h"

NS_ASSUME_NONNULL_BEGIN

@interface LofeltHaptics ()
{
    id<NativeDriver> _nativeDriver;
}

/*! @abstract       Creates an instance of LofeltHaptics that uses the given nativeDriver.
    @param error    If the initalization fails, this will be set to a valid NSError describing the error.
*/
- (instancetype)initWithNativeDriverAndReturnError:(id<NativeDriver>)nativeDriver error:(NSError **)error NS_DESIGNATED_INITIALIZER API_AVAILABLE(ios(13));

/*! @method                 createError:message:internalError:externalError
    @abstract               Creates an error to return to client code of the iOS framework given an internal error
                            that occurred within the framework or Rust core.
    @param message          The message for the error.
    @param internalError    The internal error this error is to be created from.
    @param externalError    The external error to be created (and returned to client code).
*/
- (void)createError:(NSString *)message internalError:(NSError *_Nullable *_Nullable)internalError externalError:(NSError *_Nullable *_Nullable)externalError;

/*! @method         getSDKError:error
    @abstract       Gets the latest error from the SDK.
    @param error    The error that will be filled with the latest error in the SDK.
*/
- (void)getSDKError:(NSError *_Nullable *_Nullable)error;

@end

NS_ASSUME_NONNULL_END
