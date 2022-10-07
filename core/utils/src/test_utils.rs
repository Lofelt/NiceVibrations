// Copyright (c) Meta Platforms, Inc. and affiliates.

/// Helper function to round to just 5 decimal places. This is useful to avoid floating-point
/// precision problems when comparing values.
pub fn rounded_f32(value: f32, decimal_places: u32) -> f32 {
    let f = 10_u32.pow(decimal_places) as f32;
    (value * f).round() / f
}

pub fn is_near(a: f32, b: f32, allowed_difference: f32) -> bool {
    (a - b).abs() < allowed_difference
}

/// A macro that asserts that two values are near enough to each other to be considered equal
///
/// # Example
/// ```
/// use {utils::assert_near, std::f32::consts::PI};
/// assert_near!(22.0 / 7.0_f32, PI, 2.0e-3);
/// ```
#[macro_export]
macro_rules! assert_near {
    ($a:expr, $b:expr, $allowed_difference:expr) => {{
        let a = $a;
        let b = $b;
        assert!(
            $crate::test_utils::is_near(a, b, $allowed_difference),
            "assert_near: The difference between '{}' and '{}' \
             is greater than the allowed difference of {}",
            a,
            b,
            $allowed_difference
        );
    }};
}
