/*!
 @class         MockNativeDriver
 
 @brief         The MockNativeDriver class
 
 @discussion
 @author        James Kneafsey
 @copyright     © 2020 Lofelt. All rights reserved.
 @version
 */

#import <Foundation/Foundation.h>
#import "NativeDriver.h"

NS_ASSUME_NONNULL_BEGIN

@interface MockNativeDriver: NSObject<NativeDriver>

- (MockNativeDriver *_Nonnull)init;

@end

NS_ASSUME_NONNULL_END
