//
//  LofeltHapticsTestsMacOS.m
//  LofeltHapticsTestsMacOS
//
//  Created by James Kneafsey on 09/06/2020.
//  Copyright © 2020 Lofelt. All rights reserved.
//

#import <XCTest/XCTest.h>
#import "MockNativeDriver.h"
#import "LofeltHaptics+ClassExtension.h"

@interface LofeltHapticsTestsMacOS : XCTestCase

@end

@implementation LofeltHapticsTestsMacOS

- (void)setUp {
    // Put setup code here. This method is called before the invocation of each test method in the class.
}

- (void)tearDown {
    // Put teardown code here. This method is called after the invocation of each test method in the class.
}

- (void)testPerformanceExample {
    // This is an example of a performance test case.
    [self measureBlock:^{
        // Put the code you want to measure the time of here.
    }];
}

@end
