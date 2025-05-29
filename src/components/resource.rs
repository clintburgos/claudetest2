//! Resource components

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum ResourceType {
    // Generic resources (legacy support)
    Food,
    Water,
    // Forest biome resources
    Berry,
    Mushroom,
    Nuts,
    // Desert biome resources
    CactiWater,
    DesertFruit,
    // Tundra biome resources
    IceFish,
    SnowBerry,
    // Grassland biome resources
    Seeds,
    Grass,
    // Ocean biome resources
    Seaweed,
    Shellfish,
}

impl ResourceType {
    pub fn regeneration_rate(&self) -> f32 {
        match self {
            // Generic
            ResourceType::Food => 0.1,
            ResourceType::Water => 0.2,
            // Forest
            ResourceType::Berry => 0.15,
            ResourceType::Mushroom => 0.08,
            ResourceType::Nuts => 0.12,
            // Desert
            ResourceType::CactiWater => 0.05,
            ResourceType::DesertFruit => 0.06,
            // Tundra
            ResourceType::IceFish => 0.1,
            ResourceType::SnowBerry => 0.07,
            // Grassland
            ResourceType::Seeds => 0.2,
            ResourceType::Grass => 0.25,
            // Ocean
            ResourceType::Seaweed => 0.18,
            ResourceType::Shellfish => 0.12,
        }
    }

    pub fn consumption_rate(&self) -> f32 {
        match self {
            // All food resources have similar consumption rates
            ResourceType::Food | ResourceType::Berry | ResourceType::Mushroom | 
            ResourceType::Nuts | ResourceType::DesertFruit | ResourceType::IceFish |
            ResourceType::SnowBerry | ResourceType::Seeds | ResourceType::Grass |
            ResourceType::Seaweed | ResourceType::Shellfish => 0.5,
            // Water resources
            ResourceType::Water | ResourceType::CactiWater => 0.5,
        }
    }
    
    /// Returns the nutritional value (food) and hydration value (water) of the resource
    pub fn nutritional_values(&self) -> (f32, f32) {
        match self {
            // (food_value, water_value)
            ResourceType::Food => (1.0, 0.0),
            ResourceType::Water => (0.0, 1.0),
            ResourceType::Berry => (0.8, 0.2),
            ResourceType::Mushroom => (0.9, 0.1),
            ResourceType::Nuts => (1.2, 0.0),
            ResourceType::CactiWater => (0.1, 0.9),
            ResourceType::DesertFruit => (0.6, 0.4),
            ResourceType::IceFish => (1.1, 0.1),
            ResourceType::SnowBerry => (0.7, 0.2),
            ResourceType::Seeds => (0.6, 0.0),
            ResourceType::Grass => (0.4, 0.1),
            ResourceType::Seaweed => (0.5, 0.3),
            ResourceType::Shellfish => (1.0, 0.2),
        }
    }
    
    /// Returns the color to use for rendering this resource type
    pub fn color(&self) -> Color {
        match self {
            ResourceType::Food => Color::rgb(0.8, 0.6, 0.2),
            ResourceType::Water => Color::rgb(0.2, 0.6, 0.8),
            ResourceType::Berry => Color::rgb(0.8, 0.2, 0.4),
            ResourceType::Mushroom => Color::rgb(0.7, 0.5, 0.3),
            ResourceType::Nuts => Color::rgb(0.6, 0.4, 0.2),
            ResourceType::CactiWater => Color::rgb(0.3, 0.7, 0.5),
            ResourceType::DesertFruit => Color::rgb(0.9, 0.6, 0.2),
            ResourceType::IceFish => Color::rgb(0.6, 0.7, 0.9),
            ResourceType::SnowBerry => Color::rgb(0.8, 0.8, 1.0),
            ResourceType::Seeds => Color::rgb(0.7, 0.6, 0.3),
            ResourceType::Grass => Color::rgb(0.4, 0.7, 0.3),
            ResourceType::Seaweed => Color::rgb(0.2, 0.5, 0.3),
            ResourceType::Shellfish => Color::rgb(0.9, 0.7, 0.6),
        }
    }
}

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
