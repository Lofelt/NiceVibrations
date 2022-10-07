#! /bin/sh
# Copyright (c) Meta Platforms, Inc. and affiliates. 

# This script is useful when the build fails because incompatible compilers were used, which
# produces errors like this:
# - error: Building for iOS Simulator, but the linked library 'liblofelt_sdk.a' was built for iOS.
# - error[E0514]: found crate `datamodel` compiled by an incompatible version of rustc
#   = help: please recompile that crate using this compiler (rustc 1.51.0 (2fd73fabe 2021-03-23))
# See also PD-3105.

rm -rf target/aarch64-apple-ios/ target/universal/ target/x86_64-apple-ios/
