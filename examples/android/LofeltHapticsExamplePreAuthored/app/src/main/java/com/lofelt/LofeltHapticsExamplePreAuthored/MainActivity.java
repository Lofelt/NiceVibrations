// Copyright (c) Meta Platforms, Inc. and affiliates.

package com.lofelt.LofeltHapticsExamplePreAuthored;

import android.media.MediaPlayer;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;

import com.lofelt.haptics.LofeltHaptics;

import org.apache.commons.io.IOUtils;

import java.io.InputStream;

public class MainActivity extends AppCompatActivity {
    private LofeltHaptics haptics;

    private void loadAndPlayClip(int hapticClipResourceId, int soundClipResourceId) {
        try {
            // Load haptic clip
            final InputStream stream = getResources().openRawResource(hapticClipResourceId);
            final byte[] hapticClipBytes = IOUtils.toByteArray(stream);
            haptics.load(hapticClipBytes);

            // Load audio clip
            final MediaPlayer mediaPlayer = MediaPlayer.create(this, soundClipResourceId);
            mediaPlayer.setOnCompletionListener(MediaPlayer::release);

            // Play both
            mediaPlayer.start();
            haptics.play();
        } catch (Exception e) {
            final Toast toast = Toast.makeText(getApplicationContext(), getString(R.string.playback_error_toaster, e.getMessage()), Toast.LENGTH_LONG);
            toast.show();
            e.printStackTrace();
        }
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        haptics = new LofeltHaptics(this);

        final Button achievementButton = findViewById(R.id.button_achievement);
        final Button strokeButton = findViewById(R.id.button_stroke);

        if (haptics.deviceMeetsMinimumRequirements()) {
            achievementButton.setOnClickListener(view -> loadAndPlayClip(R.raw.achievement_haptic, R.raw.achievement_audio));

            strokeButton.setOnClickListener(view -> loadAndPlayClip(R.raw.stroke_haptic, R.raw.stroke_audio));
        } else {
            achievementButton.setVisibility(View.INVISIBLE);
            strokeButton.setVisibility(View.INVISIBLE);
            final TextView textUnsupportedDevice = findViewById(R.id.textViewUnsupportedDevice);
            textUnsupportedDevice.setVisibility(View.VISIBLE);
        }
    }
}
