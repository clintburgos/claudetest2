pub mod config;
pub mod core;
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
