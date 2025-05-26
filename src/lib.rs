//! Creature Simulation Library
//!
//! A high-performance evolutionary creature simulation built with Rust and Bevy.
//! This library provides the core simulation engine for a world where digital creatures
//! evolve, learn, and interact in complex ecosystems.
//!
//! # Architecture Overview
//!
//! The simulation is organized into several key modules:
//!
//! - **`components`** - Bevy ECS components for creatures, resources, and game state
//! - **`plugins`** - Bevy plugins for simulation, rendering, UI, and more
//! - **`config`** - Configuration structures and constants for the simulation
//! - **`utils`** - Common utilities for math, randomness, and performance monitoring
//!
//! # Performance Focus
//!
//! The library is designed to handle 500+ creatures at 60+ FPS through:
//! - Spatial indexing for efficient neighbor queries
//! - Bevy ECS for cache-friendly component design
//! - Fixed timestep simulation for consistent behavior
//!
//! # Usage
//!
//! The library is designed to be used with the Bevy game engine. Core functionality
//! is exposed through plugins that can be added to a Bevy app.

pub mod components;
pub mod config;
pub mod core;
pub mod plugins;
pub mod prelude;
pub mod rendering;
pub mod simulation;
pub mod utils;

pub use bevy::math::{Vec2, Vec3};

pub type Result<T> = anyhow::Result<T>;

#[cfg(test)]
mod tests {
    #[test]
    fn basic_test() {
        assert_eq!(2 + 2, 4);
    }
}
