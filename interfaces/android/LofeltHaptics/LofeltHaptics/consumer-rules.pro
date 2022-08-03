# This file is essential for the Lofelt SDK for Android and Nice Vibrations to function correctly
# when minification with ProGuard/R8 is enabled. The reason you need this file is because
# ProGuard/R8 is limited in being able to detect exactly how the Lofelt SDK for Android works and is
# being used, and removes essential classes. The rules below prevent ProGuard/R8 from removing the
# essential classes.

# Keep the internal "Player" class, which contains functions invoked via JNI from native code in
# liblofelt_sdk.so. ProGuard/R8 is unable to detect these calls from native code.
-keep class com.lofelt.haptics.Player { *; }

# Keep the public API of the "LofeltHaptics" and "HapticPatterns" classes. This is needed if the API
# is used via JNI, like in the C# scripts of the Nice Vibrations Unity asset. ProGuard/R8 is unable
# to detect calls made via JNI.
-keep class com.lofelt.haptics.LofeltHaptics { *; }
-keep class com.lofelt.haptics.HapticPatterns { *; }
