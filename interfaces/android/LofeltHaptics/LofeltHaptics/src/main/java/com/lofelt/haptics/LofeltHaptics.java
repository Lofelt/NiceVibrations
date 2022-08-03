package com.lofelt.haptics;

import android.content.Context;
import android.os.Build;
import android.os.VibrationEffect;
import android.os.Vibrator;
import android.util.Log;

import androidx.annotation.ChecksSdkIntAtLeast;
import androidx.annotation.RequiresApi;

import java.util.Arrays;
import java.util.stream.LongStream;

// Helper class used by LofeltHaptics
//
// This contains the callbacks called from Rust, as well as the clip data loaded in loadCallback().
//
// All these things are in a separate class, and not in LofeltHaptics, because the Rust code creates
// a global reference to the player in the call to create(). That global reference is only released in
// the call to destroy(). If everything were in a single class in LofeltHaptics, the global reference
// would make it impossible for LofeltHaptics to get garbage-collected.
//
// The Rust core invokes all the callbacks from a separate thread, therefore Player should not be
// used from any other thread after it has been initialized. Using a separate thread is done for
// performance reasons: The calls to Vibrator::vibrate() and VibrationEffect.createWaveform() can
// take multiple milliseconds, and we want to avoid blocking the main thread for so long.
//
// The callbacks are marked with @SuppressWarnings("unused") to suppress an Android Studio linter
// warning about the function not being used. Android Studio doesn't know that those methods are
// called from Rust via JNI.
@RequiresApi(api = Build.VERSION_CODES.O)
class Player {

    private Vibrator vibrator;
    private VibrationEffect effect;
    private boolean clipLoaded = false;
    private boolean clipLoopingEnabled = false;
    private final Context context;

    public Player(Context context) {
        this.context = context;
    }

    @SuppressWarnings("SameParameterValue")
    private void loadCallback(long[] timings, int[] amplitudes, boolean loopEnabled) {

        clipLoaded = true;
        clipLoopingEnabled = loopEnabled;

        if (timings.length == 0 || amplitudes.length == 0 ||
                Arrays.stream(timings).allMatch(timing -> timing == 0) ||
                Arrays.stream(amplitudes).allMatch(amplitude -> amplitude == 0)) {
            // Either the timings or the amplitudes are empty or contain only zeros.
            // Calling VibrationEffect.createWaveform() throws an exception in such cases, so keep
            // the VibrationEffect as null instead.
            this.effect = null;
        } else {
            this.vibrator = (Vibrator) context.getSystemService(Context.VIBRATOR_SERVICE);

            // The glitch bug (PD-1799 / https://issuetracker.google.com/issues/171133221) only
            // happens in Android 11 (R) or earlier, therefore only use the workaround there if
            // the clip looping isn't enabled
            if (Build.VERSION.SDK_INT <= Build.VERSION_CODES.R && !clipLoopingEnabled) {
                this.effect = getPaddedEffect(timings, amplitudes);
            } else {
                this.effect = VibrationEffect.createWaveform(timings, amplitudes, getRepeatValue());
            }
        }
    }

    @SuppressWarnings("unused")
    private void playCallback() {
        if (!clipLoaded) {
            throw new RuntimeException("Unable to play, no clip loaded");
        } else if (effect == null) {
            // The clip is loaded, but empty. This can for example happen when seeking beyond the
            // end of the clip or when using an amplitude multiplication of 0.0.
            // Nothing to do in this case.
            return;
        } else if (this.vibrator == null) {
            throw new RuntimeException("Unable to play, Vibrator service unavailable");
        }

        // If the previous created `effect` was looping, it is necessary to call stop before playing
        // a new one.
        // Otherwise it will continue to play the same `effect` even if a new one was created
        stopCallback();
        vibrator.vibrate(effect);
    }

    private void stopCallback() {
        if (this.vibrator != null) {
            vibrator.cancel();
        }
    }

    @SuppressWarnings("unused")
    private void unloadCallback() {
        clipLoaded = false;
        clipLoopingEnabled = false;
        vibrator = null;
        effect = null;
    }

    // Gets repeat value based on `clipLoopingEnabled` to be used on VibrationEffect.createWaveform()
    private int getRepeatValue() {
        return clipLoopingEnabled ? 0 : -1;
    }

    @SuppressWarnings("unused")
    private void seekCallback(long[] timings, int[] amplitudes) {
        loadCallback(timings, amplitudes, false);
        stopCallback();
    }

    //Creates a new Waveform using a workaround for the glitch bug (PD-1799)
    private VibrationEffect getPaddedEffect(long[] timings, int[] amplitudes) {
        //create new arrays with "space" for padding values at the end
        long[] timings_padded = Arrays.copyOf(timings, timings.length + 1);
        int[] amplitudes_padded = Arrays.copyOf(amplitudes, amplitudes.length + 1);

        //calculate padding timing value
        long padding = calculatePaddingTiming(timings);

        //don't add padding if it's 0
        if (padding > 0) {
            timings_padded[timings.length] = padding;
            amplitudes_padded[amplitudes.length] = 1;
            return VibrationEffect.createWaveform(timings_padded, amplitudes_padded, getRepeatValue());
        } else {
            return VibrationEffect.createWaveform(timings, amplitudes, getRepeatValue());
        }

    }

    //Calculates the padding timing value (ms) added to the end of the timings array
    private long calculatePaddingTiming(long[] timings) {
        //constants for the log function to calculate
        //the padding based on duration and number of breakpoints
        final double constA = -1.53;
        final double constB = 0.39;
        final double MIN_DURATION_PADDING_MS = 100;
        final double MIN_BREAKPOINTS_PADDING = 50;

        double duration = LongStream.of(timings).sum();
        double numberOfBreakpoints = timings.length;

        //only calculate padding if number of breakpoints and duration
        //are above the minimum values that cause the glitch
        if (duration >= MIN_DURATION_PADDING_MS && numberOfBreakpoints >= MIN_BREAKPOINTS_PADDING) {
            return Math.round(4 * numberOfBreakpoints *
                    (constA + (constB * Math.log(duration))));
        } else {
            return 0;
        }

    }
}

/**
 * Plays back haptic clips authored with Lofelt Studio.
 * <p>
 * The core functionality is provided by a native library, <code>lofelt_sdk</code>, which is loaded
 * during static initialization.
 * <p>
 * After that, a {@link LofeltHaptics} instance can be created to load a haptic clip (with file extension
 * <code>.haptic</code>) and play it back. One {@link LofeltHaptics} object holds one haptic clip. To play
 * back different haptic clips, either an instance can be re-used by loading a new clip into it, or multiple
 * instances can be created, each loaded with their own haptic clips.
 * <p>
 * Due to limitations of the Android API, only one haptic clip can be played at the same time. Playing a haptic
 * clip while another is already playing will stop the previously playing clip, even if played via a
 * separate instance of {@link LofeltHaptics}. The same applies to any vibration started with Android APIs
 * such as the {@link android.os.Vibrator}.
 * <p>
 * All vibrations are triggered from a dedicated haptic thread, as the Android APIs such as
 * {@link android.os.Vibrator} can take a long time when using them for playing large
 * <code>.haptic</code> clips. That means all methods such as {@link #play()} or
 * {@link #load(byte[])} return quickly, but will not throw exceptions for all possible errors.
 * Instead, errors happening inside the haptic thread are logged to logcat.
 */
@RequiresApi(api = Build.VERSION_CODES.O)
public class LofeltHaptics {

    private final Context context;

    // Handle to the HapticsController returned by create()
    private long controllerHandle = 0;

    private static final String LOG_TAG = "lofelt-sdk";

    private native long create(Object callbackObject);

    private native void destroy(long controllerHandle);

    private native void load(long controllerHandle, byte[] clip);

    private native void play(long controllerHandle);

    private native void stop(long controllerHandle);

    private native void seek(long controllerHandle, float time);

    private native void setAmplitudeMultiplication(long controllerHandle, float amplitudeMultiplication);

    private native void loop(long controllerHandle, boolean enable);

    private native float getClipDuration(long controllerHandle);

    static {
        if (deviceSupportsMinimumPlatformVersion()) {
            Log.d(LOG_TAG, "Initializing Lofelt SDK version " + BuildConfig.VERSION_NAME);
            System.loadLibrary("lofelt_sdk");
        } else {
            Log.d(LOG_TAG, "Lofelt SDK shared library was not loaded. It only can be loaded" +
                    " from API level 26 on");
        }
    }

    /**
     * Creates a {@link LofeltHaptics} object.
     *
     * @param context The {@link android.content.Context} in which all vibrations are triggered
     * @throws RuntimeException if the initialization fails
     */
    public LofeltHaptics(Context context) {
        Log.d(LOG_TAG, "Creating LofeltHaptics instance");
        this.context = context;
        if (deviceMeetsMinimumRequirements()) {
            this.controllerHandle = create(new Player(context));
        }
    }

    @Override
    protected void finalize() throws Throwable {
        try {
            Log.d(LOG_TAG, "Finalizing LofeltHaptics instance");
            if (deviceMeetsMinimumRequirements()) {
                destroy(controllerHandle);
            }
            controllerHandle = 0;
        } catch (RuntimeException ex) {
            Log.e(LOG_TAG, "Error finalizing LofeltHaptics: " + ex);
        } finally {
            super.finalize();
        }
    }

    // Unfortunately javadoc doesn't understand @hide, so this method will appear
    // in the documentation even though we don't want users to use this method.
    // We only need it here for the Unity asset scripts.
    //
    // @SuppressWarnings("unused") is used to suppress an Android Studio linter warning about
    // this method not being used. While we indeed don't call it from any of the tests, it
    // is used by the Unity asset scripts.

    /**
     * Returns a handle to the controller of the native library.
     * <p>
     * This is an internal method, do not use.
     *
     * @return the handle to the native controller
     */
    @SuppressWarnings("unused")
    public long getControllerHandle() {
        return controllerHandle;
    }

    /**
     * Checks if the Android device platform version is supported by Lofelt Haptics
     *
     * @return true if the Android device is running Android 8 or newer; false otherwise.
     */
    @ChecksSdkIntAtLeast(api = Build.VERSION_CODES.O)
    static boolean deviceSupportsMinimumPlatformVersion() {
        return Build.VERSION.SDK_INT >= Build.VERSION_CODES.O;
    }

    /**
     * Checks if the Android device meets the minimum requirements, i.e. has amplitude control,
     * has a hardware vibrator and supports minimum platform version.
     *
     * @return true if the Android device meets the minimum requirements; false if not.
     */
    public boolean deviceMeetsMinimumRequirements() {
        if (deviceSupportsMinimumPlatformVersion()) {
            Vibrator vibrator = (Vibrator) context.getSystemService(Context.VIBRATOR_SERVICE);
            return vibrator != null && vibrator.hasVibrator() && vibrator.hasAmplitudeControl();
        } else {
            return false;
        }
    }

    /**
     * Loads a haptic clip.
     * <p>
     * Once loaded, a haptic clip can be played multiple times. It won't do anything if
     * the device doesn't meet the minimum requirements.
     *
     * @param clip the content of the <code>.haptic</code> file, which contains UTF-8 encoded JSON,
     *             without a null terminator.
     * @throws RuntimeException if loading the <code>.haptic</code> file failed, for example because
     *                          the file content is not a valid <code>.haptic</code> JSON
     */
    public void load(byte[] clip) {
        // WARNING: In Unity, this method is bypassed completely, see lofeltHapticsLoadDirect()
        // in jni_api.rs. That means any new code added here will not have an effect in Unity.
        if (deviceMeetsMinimumRequirements()) {
            load(controllerHandle, clip);
        }
    }

    /**
     * Plays back a haptic clip.
     * <p>
     * The clip needs to be loaded with {@link #load(byte[])} first.
     *
     * @throws RuntimeException if playing the haptic clip fails, for example if no clip is loaded
     *                          or if the device doesn't meet the minimum requirements
     */
    public void play() {
        if (!deviceMeetsMinimumRequirements()) {
            throw new RuntimeException("Unable to play, device doesn't meet the minimum requirements " +
                    "to play haptics");
        }
        play(controllerHandle);
    }

    /**
     * Stops playback of a currently playing haptic clip.
     * <p>
     * The call is ignored if no clip is loaded, no clip is playing or if the device doesn't
     * meet the minimum requirements.
     *
     * @throws RuntimeException if stopping the haptic clip fails
     */
    public void stop() {
        if (deviceMeetsMinimumRequirements()) {
            stop(controllerHandle);
        }
    }

    /**
     * Jumps to a time position in the haptic clip.
     * <p>
     * The clip needs to be loaded with {@link #load(byte[])} first.
     * The playback will always be stopped when this function is called.
     * If seeking beyond the end of the clip, play will not reproduce any haptics.
     * Seeking to a negative position will seek to the beginning of the clip (and playback
     * state will not change).
     * <p>
     * If looping is enabled, calling seek will have no effect.
     *
     * @param time The new position within the clip, as seconds from the beginning of the clip
     * @throws RuntimeException if seeking the haptic clip fails, for example if no clip is loaded
     */
    public void seek(float time) {
        if (deviceMeetsMinimumRequirements()) {
            seek(controllerHandle, time);
        }

    }

    /**
     * Sets the playback to repeat from the start at the end of the clip.
     * <p>
     * Changes done with this function are only applied when {@link #play()} is called.
     * When {@link #load(byte[])} is called, looping is always disabled.
     * <p>
     * Playback will always start at the beginning of the clip, even if {@link #seek(float)} was
     * used to jump to a different clip position before.
     *
     * @param enabled When true, looping is set enabled; false disables
     *                looping.
     * @throws RuntimeException if changing the looping failed
     */
    public void loop(boolean enabled) {
        if (deviceMeetsMinimumRequirements()) {
            loop(controllerHandle, enabled);
        }
    }

    /**
     * Multiplies the amplitude of every breakpoint of the clip with the given multiplication
     * factor before playing it.
     * <p>
     * In other words, this function applies a gain (for factors greater than 1.0) or an attenuation
     * (for factors less than 1.0) to the clip.
     * If the resulting amplitude of a breakpoint is greater than 1.0, it is clipped to 1.0. The
     * amplitude is clipped hard, no limiter is used.
     * <p>
     * The clip needs to be loaded with {@link #load(byte[])} first. Loading a clip resets the
     * multiplication factor back to the default of 1.0.
     * <p>
     * The new amplitudes will take effect in the next call to {@link #play()}, a call to this function
     * will not affect currently playing clips.
     *
     * @param amplitudeMultiplication The factor by which each amplitude will be multiplied. This
     *                                value is a multiplication factor, it is <i>not</i> a dB value.
     *                                The factor needs to be 0 or greater.
     * @throws RuntimeException if setting the amplitude multiplication factor fails, for example if
     *                          no clip is loaded or if the factor is outside of the valid range
     */
    public void setAmplitudeMultiplication(float amplitudeMultiplication) {
        if (deviceMeetsMinimumRequirements()) {
            setAmplitudeMultiplication(controllerHandle, amplitudeMultiplication);
        }
    }

    /**
     * Returns the duration of the loaded clip.
     *
     * @return The clip duration; 0.0 in case the clip was not loaded.
     */
    public float getClipDuration() {
        if (deviceMeetsMinimumRequirements()) {
            return getClipDuration(controllerHandle);
        } else {
            return 0.0f;
        }
    }
}
