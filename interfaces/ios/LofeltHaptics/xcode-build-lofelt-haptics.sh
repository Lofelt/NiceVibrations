# This script is invoked from Xcode during a pre-build phase for LofeltHaptics.
# Copyright (c) Meta Platforms, Inc. and affiliates.
# It relies on environment variables to be set from Xcode, so it isn't intended to be
# invoked directly, but it's exposed here as a separate file for easy editing.

# Ensure Rust build tools are in $PATH
export PATH="$HOME/.cargo/bin:$PATH"
# cd to the workspace root
cd ../../../

if [[ -n "${DEVELOPER_SDK_DIR:-}" ]]; then
  # Assume we're in Xcode, which means we're probably cross-compiling.
  # In this case, we need to add an extra library search path for build scripts and proc-macros,
  # which run on the host instead of the target.
  # (macOS Big Sur does not have linkable libraries in /usr/lib/.)
  export LIBRARY_PATH="${DEVELOPER_SDK_DIR}/MacOSX.sdk/usr/lib:${LIBRARY_PATH:-}"
fi

# Use the ios-arm64 toolchain when the ENABLE_BITCODE flag is set.
# We only need to enable bitcode when building the framework for distribution
# (see build-universal-framework.sh), otherwise the default toolchain can be used.
# Q: Why not always build with the ios-arm64 toolchain?
# A: Because it's only capable of building for iOS, which is inconvenient when building
#    for macOS or simulators.
# Q: Do we still need to use the ios-arm64 toolchain given that Rust embeds bitcode for
#    iOS targets since 1.46.0?
# A: Not sure! Ditto released a new toolchain when 1.46.0 came out (we're currently using
#    the toolchain built with 1.43.0), which works with stable Rust rather than nightly.
#    We should update to this new toolchain, but we can also investigate if it's necessary
#    to use a special toolchain at all, and instead pin the version of Rust that we use to
#    a version that's known to generate App-Store compatible bitcode.
# Q: Why do we use cargo-lipo at all given that build-universal-framework.sh generates a
#    distribution framework using lipo directly?
# A: cargo-lipo does a good job of reading Xcode flags and building Rust code for the
#    correct platform and configuration. Given the range of platforms and configurations
#    that need to be targeted (which is made more complicated with the introduction
#    of ARM-based Macs), using cargo-lipo for now is simpler than maintaining the
#    equivalent build script.
if [ "${ENABLE_BITCODE}" = "YES" ]; then
  echo "Building rust lib with bitcode"
  RUSTFLAGS="-C embed-bitcode" cargo +ios-arm64-nightly-2021-10-05 lipo --xcode-integ --package api
else
  echo "Building rust lib without bitcode"
  cargo lipo --xcode-integ --package api
fi
