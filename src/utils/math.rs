//! Mathematical utilities and helper functions.

use crate::Vec2;

/// Linearly interpolates between two values.
///
/// # Arguments
/// * `a` - Start value
/// * `b` - End value
/// * `t` - Interpolation factor (0.0 to 1.0)
///
/// # Returns
/// Interpolated value between a and b
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Linearly interpolates between two vectors.
#[inline]
pub fn lerp_vec2(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t
}

/// Clamps a value between min and max.
///
/// More efficient than calling f32::clamp when you know min < max.
#[inline]
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    debug_assert!(min <= max, "min must be less than or equal to max");
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Maps a value from one range to another.
///
/// # Arguments
/// * `value` - Value to map
/// * `in_min` - Input range minimum
/// * `in_max` - Input range maximum
/// * `out_min` - Output range minimum
/// * `out_max` - Output range maximum
///
/// # Algorithm
/// 1. Normalize the input value to [0, 1] range: (value - in_min) / (in_max - in_min)
/// 2. Scale to output range: normalized * (out_max - out_min)
/// 3. Shift to output minimum: + out_min
///
/// Example: map_range(75, 50, 100, 0, 1) = 0.5
/// - Normalize: (75-50)/(100-50) = 25/50 = 0.5
/// - Scale: 0.5 * (1-0) = 0.5
/// - Shift: 0.5 + 0 = 0.5
#[inline]
pub fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let normalized = (value - in_min) / (in_max - in_min);
    out_min + normalized * (out_max - out_min)
}

/// Smoothstep interpolation for smooth transitions.
///
/// Returns a smooth curve that eases in and out.
///
/// # Algorithm
/// Uses the cubic Hermite interpolation formula: 3t² - 2t³
/// This creates an S-shaped curve with zero derivatives at t=0 and t=1
/// making it ideal for smooth transitions without sudden velocity changes
///
/// Properties:
/// - f(0) = 0, f(1) = 1
/// - f'(0) = 0, f'(1) = 0 (smooth start and end)
/// - Monotonic increasing for t ∈ [0,1]
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    // First normalize x to [0, 1] range and clamp
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    // Apply smoothstep formula: 3t² - 2t³
    // This can also be written as: t² * (3 - 2t)
    t * t * (3.0 - 2.0 * t)
}

/// Calculates the squared distance between two points.
///
/// More efficient than distance when you only need to compare distances.
#[inline]
pub fn distance_squared(a: Vec2, b: Vec2) -> f32 {
    (b - a).length_squared()
}

/// Checks if a point is within a circle.
#[inline]
pub fn point_in_circle(point: Vec2, center: Vec2, radius: f32) -> bool {
    distance_squared(point, center) <= radius * radius
}

/// Normalizes an angle to the range [0, 2π].
#[inline]
pub fn normalize_angle(angle: f32) -> f32 {
    let two_pi = std::f32::consts::PI * 2.0;
    let mut normalized = angle % two_pi;
    if normalized < 0.0 {
        normalized += two_pi;
    }
    normalized
}

/// Calculates the shortest angular distance between two angles.
///
/// # Algorithm
/// Handles the circular nature of angles by finding the shortest path
/// around the circle. Since angles wrap at 2π, the difference between
/// 350° and 10° is 20°, not 340°.
///
/// 1. Calculate raw difference: to - from
/// 2. Normalize to [-π, π] range to handle wraparound
/// 3. Return absolute value for unsigned distance
///
/// Example: angle_difference(350°, 10°)
/// - Raw diff: 10° - 350° = -340°
/// - Add 2π: -340° + 360° = 20°
/// - Result: |20°| = 20°
#[inline]
pub fn angle_difference(from: f32, to: f32) -> f32 {
    let mut diff = to - from;
    let pi = std::f32::consts::PI;

    // Normalize to [-pi, pi] range
    // This finds the shortest path around the circle
    while diff > pi {
        diff -= 2.0 * pi;
    }
    while diff < -pi {
        diff += 2.0 * pi;
    }

    diff.abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(lerp(-10.0, 10.0, 0.5), 0.0);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn test_map_range() {
        assert_eq!(map_range(0.5, 0.0, 1.0, 0.0, 100.0), 50.0);
        assert_eq!(map_range(75.0, 50.0, 100.0, 0.0, 1.0), 0.5);
    }

    #[test]
    fn test_smoothstep() {
        assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);
        assert!(smoothstep(0.0, 1.0, 0.5) > 0.4 && smoothstep(0.0, 1.0, 0.5) < 0.6);
    }

    #[test]
    fn test_angle_difference() {
        use std::f32::consts::PI;

        assert!((angle_difference(0.0, PI) - PI).abs() < 0.001);
        assert!((angle_difference(0.0, -PI) - PI).abs() < 0.001);
        assert!((angle_difference(PI * 0.5, PI * 1.5) - PI).abs() < 0.001);
    }
}
