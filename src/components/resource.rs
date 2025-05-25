//! Resource components

use bevy::prelude::*;
use crate::simulation::ResourceType;

/// Marker component for resource entities
#[derive(Component, Debug, Default)]
pub struct ResourceMarker;

/// Type of resource
#[derive(Component, Debug, Clone)]
pub struct ResourceTypeComponent(pub ResourceType);

/// Current amount of resource available
#[derive(Component, Debug, Clone)]
pub struct ResourceAmount {
    pub current: f32,
    pub max: f32,
}

impl ResourceAmount {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
    
    /// Consumes up to the specified amount, returns actual amount consumed
    pub fn consume(&mut self, amount: f32) -> f32 {
        let consumed = amount.min(self.current);
        self.current -= consumed;
        consumed
    }
    
    /// Regenerates the resource by the specified amount
    pub fn regenerate(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
    
    pub fn is_depleted(&self) -> bool {
        self.current <= 0.0
    }
    
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }
    
    pub fn percentage(&self) -> f32 {
        if self.max > 0.0 {
            self.current / self.max
        } else {
            0.0
        }
    }
}