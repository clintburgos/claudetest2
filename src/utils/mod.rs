//! Utility functions and helpers for the creature simulation.
//!
//! This module contains common functionality used across the simulation
//! including math helpers, random number generation utilities, and
//! performance measurement tools.

pub mod math;
pub mod perf;
pub mod random;

pub use math::*;
pub use perf::*;
pub use random::*;
