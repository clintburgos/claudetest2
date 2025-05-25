//! Utility functions and helpers for the creature simulation.
//! 
//! This module contains common functionality used across the simulation
//! including math helpers, random number generation utilities, and
//! performance measurement tools.

pub mod math;
pub mod random;
pub mod perf;

pub use math::*;
pub use random::*;
pub use perf::*;