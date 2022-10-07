// Copyright (c) Meta Platforms, Inc. and affiliates.

package com.lofelt.haptics;

import android.content.Context;
import android.os.Build;
import android.os.Vibrator;

import androidx.annotation.RequiresApi;

/**
 * This class is responsible for playback of basic haptic patterns create at runtime. It doesn't
 * use the <code>lofelt_sdk</code> native library.
 * <p>
 * Android devices that don't meet the minimum requirements of LofeltHaptics class can make use of
 * this.
 */
@RequiresApi(api = Build.VERSION_CODES.JELLY_BEAN_MR1)
public class HapticPatterns {

    final private Vibrator vibrator;

    public HapticPatterns(Context context) {
        this.vibrator = (Vibrator) context.getSystemService(Context.VIBRATOR_SERVICE);
    }

    /**
     * Plays back a pattern of on/off vibrations with the maximum amplitude.
     * <p>
     * Can be used as a fallback for Android versions that don't support the minimum requirements,
     * of LofeltHaptics.
     * <p>
     *
     * @param patternPoints Each value represents a point in time. Two adjacent points define the
     *                      duration in seconds of turning the vibration motor on/off. The first 2
     *                      adjacent points define how long the motor is on, the next 2 define how
     *                      long it is off. Subsequent groups of 2 adjacent points alternate between
     *                      durations of when to turn the motor off or on.
     *                      E.g. if patternPoints = { 0.0, 0.2, 0.3, 0.4, 0.6, 0.7}, this will
     *                      translate to a vibration motor on/off pattern of
     *                      { 0.2-0.0= 0.2s ON, 0.3-0.2=0.1s OFF, 0.1s ON, 0.2s OFF, 0.1 ON}
     *                      <p>
     *                      Needs to have at least 2 points in the array.
     */
    public void playMaximumAmplitudePattern(float[] patternPoints) {
        if (vibrator != null && vibrator.hasVibrator() && patternPoints.length > 1) {
            long[] timings = new long[patternPoints.length];
            // playback starts with the motor turned off for 0ms
            timings[0] = 0;
            for (int i = 1; i < patternPoints.length; i++) {
                long diff = (long) ((patternPoints[i] - patternPoints[i - 1]) * 1000.0f);
                timings[i] = diff;
            }
            vibrator.vibrate(timings, -1);
        }
    }

    /**
     * Stops the playback of a pattern played with {@link #playMaximumAmplitudePattern(float[])}.
     */
    @SuppressWarnings("unused")
    public void stopPattern() {
        if (vibrator != null && vibrator.hasVibrator()) {
            vibrator.cancel();
        }
    }
}
