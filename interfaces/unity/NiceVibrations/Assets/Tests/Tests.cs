// Copyright (c) Meta Platforms, Inc. and affiliates. 

using NUnit.Framework;
using Lofelt.NiceVibrations;
using System.Threading;
using System.Globalization;
using System;
using UnityEngine;

namespace NiceVibrationTests
{
    public class Tests
    {
        // This tests SDK-106 and ensures that the haptic clip JSON for presets is correct,
        // even when using a locale in which the decimal separator is a comma.
        [Test]
        public void PlayPresetLocale()
        {
            Thread.CurrentThread.CurrentCulture = new CultureInfo("de-DE");
            String actualJson = System.Text.Encoding.UTF8.GetString(HapticPatterns.Medium.jsonClip);
            String expectedJson = (Resources.Load("MediumPreset.haptic") as TextAsset).text;
            Assert.AreEqual(expectedJson, actualJson);
        }
    }
}
