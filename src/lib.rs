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
//! - **`config`** - Configuration structures and constants for the simulation
//! - **`core`** - Fundamental systems including entities, world management, and events
//! - **`simulation`** - Creature behaviors, health, needs, and resource management
//! - **`systems`** - ECS systems that drive the simulation forward
//! - **`utils`** - Common utilities for math, randomness, and performance monitoring
//!
//! # Performance Focus
//!
//! The library is designed to handle 5000+ creatures at 60+ FPS through:
//! - Spatial indexing for efficient neighbor queries
//! - Parallel processing with Rayon
//! - Cache-friendly ECS component design
//! - LOD (Level of Detail) systems for scalable complexity
//!
//! # Usage
//!
//! The library is designed to be used with the Bevy game engine. Core functionality
//! is exposed through plugins that can be added to a Bevy app.

pub mod components;
pub mod config;
pub mod core;
pub mod plugins;
pub mod simulation;
pub mod systems;
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
