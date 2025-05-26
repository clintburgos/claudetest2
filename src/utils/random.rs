//! Random number generation utilities.
//!
//! Provides convenience functions for common random operations
//! used throughout the simulation. Currently uses simple
//! deterministic methods for Phase 1.

use crate::Vec2;

/// Generates a pseudo-random direction vector.
///
/// This is a deterministic function based on position for Phase 1.
/// In production, this should use a proper RNG.
///
/// # Arguments
/// * `seed` - Seed vector (typically a position)
///
/// # Returns
/// A normalized direction vector
pub fn random_direction(seed: Vec2) -> Vec2 {
    let angle = (seed.x * 12.9898 + seed.y * 78.233).sin() * 43_758.547;
    let angle = angle.fract() * std::f32::consts::TAU;
    Vec2::new(angle.cos(), angle.sin())
}

/// Generates a random point within a circle.
///
/// # Arguments
/// * `center` - Center of the circle
/// * `radius` - Radius of the circle
/// * `seed` - Random seed
///
/// # Returns
/// A random point within the specified circle
pub fn random_point_in_circle(center: Vec2, radius: f32, seed: f32) -> Vec2 {
    // Simple deterministic approach for Phase 1
    let angle = seed * std::f32::consts::TAU;
    let r = ((seed * 7.919).fract() * radius).sqrt();

    center + Vec2::new(angle.cos() * r, angle.sin() * r)
}

/// Generates a random value between 0 and 1.
///
/// Deterministic based on seed for Phase 1.
#[inline]
pub fn random_01(seed: f32) -> f32 {
    ((seed * 12.9898).sin() * 43_758.547).fract().abs()
}

/// Generates a random value in a range.
///
/// # Arguments
/// * `min` - Minimum value (inclusive)
/// * `max` - Maximum value (exclusive)
/// * `seed` - Random seed
#[inline]
pub fn random_range(min: f32, max: f32, seed: f32) -> f32 {
    min + random_01(seed) * (max - min)
}

/// Returns true with the given probability.
///
/// # Arguments
/// * `probability` - Chance of returning true (0.0 to 1.0)
/// * `seed` - Random seed
#[inline]
pub fn random_chance(probability: f32, seed: f32) -> bool {
    random_01(seed) < probability
}

/// Selects a random item from a slice based on weights.
///
/// # Arguments
/// * `items` - Slice of items to choose from
/// * `weights` - Weights for each item (must be same length as items)
/// * `seed` - Random seed
///
/// # Returns
/// Index of the selected item, or None if inputs are invalid
pub fn weighted_random_choice(weights: &[f32], seed: f32) -> Option<usize> {
    if weights.is_empty() {
        return None;
    }

    let total: f32 = weights.iter().sum();
    if total <= 0.0 {
        return None;
    }

    let mut random = random_01(seed) * total;

    for (i, &weight) in weights.iter().enumerate() {
        random -= weight;
        if random <= 0.0 {
            return Some(i);
        }
    }

    // Fallback (should not happen with valid inputs)
    Some(weights.len() - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_direction() {
        let dir1 = random_direction(Vec2::new(0.0, 0.0));
        let dir2 = random_direction(Vec2::new(1.0, 0.0));

        // Should be normalized
        assert!((dir1.length() - 1.0).abs() < 0.001);
        assert!((dir2.length() - 1.0).abs() < 0.001);

        // Different seeds should give different directions
        assert_ne!(dir1, dir2);
    }

    #[test]
    fn test_random_01() {
        for i in 0..100 {
            let val = random_01(i as f32);
            assert!((0.0..=1.0).contains(&val));
        }
    }

    #[test]
    fn test_random_range() {
        for i in 0..100 {
            let val = random_range(10.0, 20.0, i as f32);
            assert!((10.0..20.0).contains(&val));
        }
    }

    #[test]
    fn test_weighted_random_choice() {
        let weights = vec![1.0, 2.0, 3.0];

        // Should always return valid index
        for i in 0..100 {
            let choice = weighted_random_choice(&weights, i as f32);
            assert!(choice.is_some());
            assert!(choice.unwrap() < weights.len());
        }

        // Empty weights should return None
        assert_eq!(weighted_random_choice(&[], 0.0), None);

        // All zero weights should return None
        assert_eq!(weighted_random_choice(&[0.0, 0.0], 0.0), None);
    }

    #[test]
    fn test_random_point_in_circle() {
        let center = Vec2::new(100.0, 100.0);
        let radius = 50.0;

        // Test multiple points
        for i in 0..100 {
            let point = random_point_in_circle(center, radius, i as f32 * 0.1);
            let distance = (point - center).length();

            // Point should be within the circle
            assert!(
                distance <= radius,
                "Point at distance {} exceeds radius {}",
                distance,
                radius
            );
        }

        // Test that points are distributed (not all at center)
        let p1 = random_point_in_circle(center, radius, 0.1);
        let p2 = random_point_in_circle(center, radius, 0.5);
        let p3 = random_point_in_circle(center, radius, 0.9);

        assert_ne!(p1, p2);
        assert_ne!(p2, p3);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_random_chance() {
        // Always true
        for i in 0..10 {
            assert!(random_chance(1.0, i as f32));
        }

        // Never true
        for i in 0..10 {
            assert!(!random_chance(0.0, i as f32));
        }

        // Test approximate probability
        let mut successes = 0;
        let trials = 1000;
        let probability = 0.3;

        for i in 0..trials {
            if random_chance(probability, i as f32 * 0.01) {
                successes += 1;
            }
        }

        let actual_rate = successes as f32 / trials as f32;
        // Should be roughly 30% (with some variance)
        assert!(actual_rate > 0.25 && actual_rate < 0.35);
    }

    #[test]
    fn test_deterministic_behavior() {
        // Same seed should produce same results
        let seed = Vec2::new(42.0, 42.0);
        let dir1 = random_direction(seed);
        let dir2 = random_direction(seed);
        assert_eq!(dir1, dir2);

        let val1 = random_01(42.0);
        let val2 = random_01(42.0);
        assert_eq!(val1, val2);

        let range1 = random_range(10.0, 20.0, 42.0);
        let range2 = random_range(10.0, 20.0, 42.0);
        assert_eq!(range1, range2);
    }

    #[test]
    fn test_weighted_distribution() {
        // Test that higher weights are selected more often
        let weights = vec![1.0, 9.0]; // 10% vs 90%
        let mut counts = vec![0, 0];

        for i in 0..1000 {
            if let Some(choice) = weighted_random_choice(&weights, i as f32 * 0.01) {
                counts[choice] += 1;
            }
        }

        // Second choice should be selected much more often
        assert!(counts[1] > counts[0] * 5);
    }
}
