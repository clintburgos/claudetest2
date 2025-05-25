//! Health components

use bevy::prelude::*;

/// Health component for creatures
#[derive(Component, Debug, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
    
    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
    
    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
    
    pub fn is_dead(&self) -> bool {
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

impl Default for Health {
    fn default() -> Self {
        Self::new(100.0)
    }
}