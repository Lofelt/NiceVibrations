// Copyright (c) Meta Platforms, Inc. and affiliates.

package com.lofelt.haptics;

import android.content.Context;
import android.os.Build;

import androidx.test.ext.junit.runners.AndroidJUnit4;
import androidx.test.platform.app.InstrumentationRegistry;

import com.lofelt.haptics.test.R;

import org.apache.commons.io.IOUtils;
import org.junit.Test;
import org.junit.runner.RunWith;

import java.io.IOException;
import java.io.InputStream;
import java.lang.ref.WeakReference;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

@RunWith(AndroidJUnit4.class)
public class LofeltHapticsInstrumentedTest {
    private boolean isEmulator() {
        return (Build.BRAND.startsWith("generic") && Build.DEVICE.startsWith("generic"))
                || Build.FINGERPRINT.startsWith("generic")
                || Build.FINGERPRINT.startsWith("unknown")
                || Build.HARDWARE.contains("goldfish")
                || Build.HARDWARE.contains("ranchu")
                || Build.MODEL.contains("google_sdk")
                || Build.MODEL.contains("Emulator")
                || Build.MODEL.contains("Android SDK built for x86")
                || Build.MANUFACTURER.contains("Genymotion")
                || Build.PRODUCT.contains("sdk_google")
                || Build.PRODUCT.contains("google_sdk")
                || Build.PRODUCT.contains("sdk")
                || Build.PRODUCT.contains("sdk_x86")
                || Build.PRODUCT.contains("vbox86p")
                || Build.PRODUCT.contains("emulator")
                || Build.PRODUCT.contains("simulator");
    }

    // Tests that a .haptic clip can be played
    @Test
    public void playClip() throws InterruptedException, IOException {
        // This is a device-only test that should not run on an emulator.
        if (isEmulator()) {
            return;
        }

        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        lofeltHaptics.play();

        // Give the clip some time to finish before exiting the test.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(2300);
    }

    // Tests that amplitude multiplication works
    @Test
    public void setAmplitudeMultiplication() throws InterruptedException, IOException {
        // This is a device-only test that should not run on an emulator.
        if (isEmulator()) {
            return;
        }

        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);

        // Play once with normal amplitude, then once with a lower amplitude. That way the two plays
        // can easily be perceptually compared.
        lofeltHaptics.play();
        Thread.sleep(2300);
        lofeltHaptics.setAmplitudeMultiplication(0.25f);
        lofeltHaptics.play();
        Thread.sleep(2300);
    }

    // Tests that emphasis can be felt.
    // The clip has a constant amplitude of 0.5 lasting for 2 seconds.
    // At 1.0 seconds, the breakpoint has emphasis.
    @Test
    public void playEmphasis() throws InterruptedException, IOException {
        // This is a device-only test that should not run on an emulator.
        if (isEmulator()) {
            return;
        }

        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.emphasis);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        lofeltHaptics.play();

        // Give the clip some time to finish before exiting the test.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(2500);
    }

    // Verifies a LofeltHaptics object can be properly garbage-collected
    @Test
    public void garbageCollection() {
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);
        WeakReference<LofeltHaptics> weakRef = new WeakReference<>(lofeltHaptics);
        //noinspection UnusedAssignment
        lofeltHaptics = null;

        // This is just a request to the garbage collector, it is not forced to do a collection.
        // In theory this can make the test unstable, in practice it seems to work.
        System.runFinalization();
        System.gc();

        assertNull(weakRef.get());
    }


    // Tests that a .haptic clip can be stopped while playing
    @Test
    public void stopClip() throws InterruptedException, IOException {
        // This is a device-only test that should not run on an emulator.
        if (isEmulator()) {
            return;
        }

        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        lofeltHaptics.play();
        Thread.sleep(200);
        lofeltHaptics.stop();

        // This is just for manual testing, when wanting to check if the vibrations
        // actually stop before the test exits.
        Thread.sleep(2100);
    }

    // Tests that loading an invalid .haptic clip raises an exception
    @Test
    public void loadInvalidClip() throws IOException {
        if (isEmulator()) {
            return;
        }
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.invalid);
        byte[] clip = IOUtils.toByteArray(stream);

        RuntimeException exception = assertThrows(
            RuntimeException.class,
            () -> lofeltHaptics.load(clip));
        assertTrue(exception.getMessage() != null && exception.getMessage().contains("missing field `amplitude`"));
    }

    // Tests that attempting to play without loading a clip first will fail
    @Test
    public void playWithoutLoading() {
        // This is a device-only test that should not run on an emulator.
        if (isEmulator()) {
            return;
        }

        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        RuntimeException exception = assertThrows(
            RuntimeException.class,
            lofeltHaptics::play);
        assertEquals(exception.getMessage(), "Unable to play, no clip loaded.");
    }

    // Tests that stop() can be called when no clip is loaded
    @Test
    public void stopWithoutLoading() {
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);
        lofeltHaptics.stop();
    }

    // Tests that stop() can be called when no clip is playing
    @Test
    public void stopWithoutPlaying() throws IOException {
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        lofeltHaptics.stop();
    }

    // Test that seek works and doesn't play from the start of `clip`
    @Test
    public void seekClip() throws IOException, InterruptedException {
        // This is a device-only test that should not run on an emulator.
        if (isEmulator()) {
            return;
        }

        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        lofeltHaptics.seek(0.3f);
        lofeltHaptics.play();

        // Give the clip some time to finish before exiting the test.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(2300);
    }

    // Tests that when seeking after the end of the clip, seek doesn't return an error
    // and play doesn't produce any haptics
    @Test
    public void seekLongerThanClip() throws IOException, InterruptedException {
        if (isEmulator()) {
            return;
        }

        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        lofeltHaptics.seek(10f);
        lofeltHaptics.play();

        // Give the clip some time to finish before exiting the test.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(2300);
    }

    // Tests that going seeking back and forward works
    @Test
    public void seekForwardAndBackward() throws IOException, InterruptedException {
        if (isEmulator()) {
            return;
        }

        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        // Seek forward and play
        lofeltHaptics.seek(0.4f);
        lofeltHaptics.play();
        // Give the clip some time to play before the next seek
        Thread.sleep(1500);
        // Seek backward and play
        lofeltHaptics.seek(0.1f);
        lofeltHaptics.play();
        // Give the clip some time to finish before exiting the test.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(2300);
    }

    // Tests that a runtime error is thrown when seek is called without load
    @Test
    public void seekWithoutLoadFails() throws InterruptedException {
        if (isEmulator()) {
            return;
        }

        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        RuntimeException exception = assertThrows(
            RuntimeException.class,
            () -> lofeltHaptics.seek(0.2f));
        assertEquals(exception.getMessage(), "Unable to seek, no clip loaded.");


        // Give the clip some time to finish before exiting the test.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(2300);
    }

    // Tests that when seek is triggered during playback, playback will stop
    @Test
    public void seekAfterPlayStops() throws IOException, InterruptedException {
        if (isEmulator()) {
            return;
        }

        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        // Seek forward and play
        lofeltHaptics.play();
        // Give the clip some time to play before the next seek that will stop playback
        Thread.sleep(500);
        // Seek will stop the clip and should be verified during manual testing
        lofeltHaptics.seek(0.1f);
        // Give the clip some time to finish before exiting the test.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(2300);
    }

    // Tests that seeking to a negative value will make the clip play from
    // the start
    @Test
    public void seekNegative() throws IOException, InterruptedException {
        if (isEmulator()) {
            return;
        }
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        lofeltHaptics.seek(-1f);
        lofeltHaptics.play();

        // Give the clip some time to finish before exiting the test.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(2300);
    }

    //Tests that a clip repeats more than 1 time
    @Test
    public void loopClip() throws InterruptedException, IOException {
        if (isEmulator()) {
            return;
        }
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);


        lofeltHaptics.load(clip);
        lofeltHaptics.loop(true);
        lofeltHaptics.play();
        // Give the clip some time to playback before exiting the test.
        // Check that playback repeats.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(8000);
        lofeltHaptics.stop();

    }

    //Tests that clip doesn't loop when `loop(false)`
    @Test
    public void doNotLoopClip() throws InterruptedException, IOException {
        if (isEmulator()) {
            return;
        }
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);


        lofeltHaptics.load(clip);
        lofeltHaptics.loop(false);
        lofeltHaptics.play();
        // Give the clip some time to play before exiting the test.
        // Check that it doesn't repeat.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(5000);
        lofeltHaptics.stop();

    }


    //Tests that the loop settings persist after `stop()` is called
    @Test
    public void loopAfterStopAndPlay() throws InterruptedException, IOException {

        if (isEmulator()) {
            return;
        }
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);


        lofeltHaptics.load(clip);
        lofeltHaptics.loop(true);
        lofeltHaptics.play();
        // Give the clip some time to play until it gets stopped.
        Thread.sleep(2000);
        lofeltHaptics.stop();
        Thread.sleep(3000);
        lofeltHaptics.play();
        // Give the clip some time to loop the playback before exiting the test.
        // Check that playback repeats.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(5000);
    }

    //Tests that after `load()` the clip won't loop, even if it did before `load()`.
    @Test
    public void loopDisabledAfterLoad() throws IOException, InterruptedException {
        if (isEmulator()) {
            return;
        }
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        lofeltHaptics.loop(true);
        lofeltHaptics.play();
        // Give the clip some time to play
        Thread.sleep(1000);
        lofeltHaptics.stop();
        Thread.sleep(1000);
        lofeltHaptics.load(clip);
        lofeltHaptics.play();
        // Give the clip some time to play before exiting the test.
        // Check that it doesn't repeat.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(5000);

    }

    // Tests that playing a new clip while the previous one was playing, plays the correct clip
    // without looping
    @Test
    public void loopStopsWhenPlayingNewClip() throws IOException, InterruptedException {
        if (isEmulator()) {
            return;
        }
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream streamAchievementClip = context.getResources().openRawResource(R.raw.clip);
        InputStream streamStrokeClip = context.getResources().openRawResource(R.raw.stroke);

        byte[] achievementClip = IOUtils.toByteArray(streamAchievementClip);
        byte[] strokeClip = IOUtils.toByteArray(streamStrokeClip);

        lofeltHaptics.load(achievementClip);
        lofeltHaptics.loop(true);
        lofeltHaptics.play();
        // Give the first clip some time to play and repeat
        Thread.sleep(5000);
        lofeltHaptics.load(strokeClip);
        lofeltHaptics.play();
        // Check that the second clip feels different than the first clip.
        // Check that the second clip doesn't repeat.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.
        Thread.sleep(5000);

    }


    // Tests that looping a sought clip will always play and loop from start to end
    @Test
    public void loopBeforeSeekPlaysFromStartAndNotSeekPosition() throws IOException, InterruptedException {
        if (isEmulator()) {
            return;
        }
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);

        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        lofeltHaptics.loop(true);
        lofeltHaptics.seek(0.8f);
        lofeltHaptics.play();
        // Give the clip some time to play and loop
        Thread.sleep(5000);
        // Check that it loops from start.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.

    }

    // Tests that that seeking in clip with looping enabled will always play and loop from start
    // to end
    @Test
    public void loopAfterSeekPlaysFromStartAndNotSeekPosition() throws IOException, InterruptedException {
        if (isEmulator()) {
            return;
        }
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);

        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);
        lofeltHaptics.seek(0.8f);
        lofeltHaptics.loop(true);
        lofeltHaptics.play();
        // Give the clip some time to play and loop
        Thread.sleep(5000);
        // Check that it loops from start.
        // This is just for manual testing, when wanting to check if the vibrations
        // feel right.

    }

    // Tests that the returned duration value of the loaded clip matches
    // the duration of clip.haptic
    @Test
    public void getLoadedClipDuration() throws IOException {
        if (isEmulator()) {
            return;
        }
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        LofeltHaptics lofeltHaptics = new LofeltHaptics(context);

        InputStream stream = context.getResources().openRawResource(R.raw.clip);
        byte[] clip = IOUtils.toByteArray(stream);

        lofeltHaptics.load(clip);

        float result = lofeltHaptics.getClipDuration();
        float expected = (float) 2.2969615;

        assertEquals(0, Float.compare(result, expected));
    }

    // Tests the check of minimum supported version, depending on which version of platform
    // the test runs
    @Test
    public void checkMinimumSupportedVersion() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            assertTrue(LofeltHaptics.deviceSupportsMinimumPlatformVersion());
        } else {
            assertFalse(LofeltHaptics.deviceSupportsMinimumPlatformVersion());
        }
    }
}
