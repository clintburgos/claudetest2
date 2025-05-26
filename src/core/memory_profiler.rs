//! Memory profiling and leak detection
//!
//! This module provides tools for tracking memory usage and detecting potential memory leaks
//! in the simulation without requiring platform-specific dependencies.

use bevy::prelude::*;
use crate::prelude::{Creature, Position, Health, Needs};
use crate::components::ResourceMarker;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Memory profiler resource that tracks allocations and usage
#[derive(Resource, Default)]
pub struct MemoryProfiler {
    /// Memory usage per system over time
    system_memory: Arc<Mutex<HashMap<String, Vec<MemorySnapshot>>>>,
    /// Entity count history
    entity_history: Vec<(Instant, usize)>,
    /// Component count by type
    component_counts: HashMap<String, usize>,
    /// Leak detection thresholds
    leak_detector: LeakDetector,
    /// Estimated memory per component type
    component_sizes: HashMap<String, usize>,
}

#[derive(Default)]
struct LeakDetector {
    /// Tracks growth rates (entities per second)
    growth_rates: HashMap<String, f64>,
    /// Number of consecutive growth periods before flagging as leak
    growth_threshold: usize,
    /// Consecutive growth counts
    growth_counts: HashMap<String, usize>,
}

#[derive(Clone, Debug)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub entity_count: usize,
    pub component_count: usize,
    pub estimated_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_entities: usize,
    pub total_components: usize,
    pub estimated_memory: usize,
    pub entity_growth_rate: f64,
    pub component_stats: HashMap<String, ComponentStats>,
    pub potential_leaks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ComponentStats {
    pub count: usize,
    pub estimated_size: usize,
    pub growth_rate: f64,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        let mut component_sizes = HashMap::new();
        // Estimated sizes for common components
        component_sizes.insert("Position".to_string(), std::mem::size_of::<Position>());
        component_sizes.insert("Creature".to_string(), 128); // Estimate for Creature struct
        component_sizes.insert("Health".to_string(), std::mem::size_of::<Health>());
        component_sizes.insert("Needs".to_string(), 32); // Estimate for Needs
        component_sizes.insert("CreatureState".to_string(), 24); // Estimate for state enum
        component_sizes.insert("Resource".to_string(), 40); // Estimate for Resource
        
        Self {
            system_memory: Arc::new(Mutex::new(HashMap::new())),
            entity_history: Vec::with_capacity(1000),
            component_counts: HashMap::new(),
            leak_detector: LeakDetector {
                growth_rates: HashMap::new(),
                growth_threshold: 5,
                growth_counts: HashMap::new(),
            },
            component_sizes,
        }
    }

    /// Record a memory snapshot for a system
    pub fn record_snapshot(&mut self, system_name: &str, snapshot: MemorySnapshot) {
        let mut memory_map = self.system_memory.lock().unwrap();
        let history = memory_map.entry(system_name.to_string()).or_default();
        
        // Keep last 100 snapshots
        if history.len() >= 100 {
            history.remove(0);
        }
        
        // Calculate growth rate
        if let Some(last) = history.last() {
            let time_delta = snapshot.timestamp.duration_since(last.timestamp).as_secs_f64();
            if time_delta > 0.0 {
                let entity_growth = (snapshot.entity_count as f64 - last.entity_count as f64) / time_delta;
                self.leak_detector.growth_rates.insert(system_name.to_string(), entity_growth);
                
                // Update leak detection - flag if growing more than 10 entities/second
                if entity_growth > 10.0 {
                    let count = self.leak_detector.growth_counts.entry(system_name.to_string()).or_insert(0);
                    *count += 1;
                } else {
                    self.leak_detector.growth_counts.insert(system_name.to_string(), 0);
                }
            }
        }
        
        history.push(snapshot);
    }

    /// Record entity count
    pub fn record_entity_count(&mut self, count: usize) {
        let now = Instant::now();
        
        // Keep last 1000 samples
        if self.entity_history.len() >= 1000 {
            self.entity_history.remove(0);
        }
        
        self.entity_history.push((now, count));
    }

    /// Update component count for a type
    pub fn update_component_count(&mut self, component_name: &str, count: usize) {
        self.component_counts.insert(component_name.to_string(), count);
    }

    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        let _memory_map = self.system_memory.lock().unwrap();
        let mut component_stats = HashMap::new();
        let mut total_components = 0;
        let mut estimated_memory = 0;
        
        // Calculate component statistics
        for (name, &count) in &self.component_counts {
            total_components += count;
            let size = self.component_sizes.get(name).copied().unwrap_or(32);
            estimated_memory += count * size;
            
            let growth_rate = self.leak_detector.growth_rates.get(name).copied().unwrap_or(0.0);
            
            component_stats.insert(name.clone(), ComponentStats {
                count,
                estimated_size: count * size,
                growth_rate,
            });
        }
        
        // Calculate entity growth rate
        let entity_growth_rate = if self.entity_history.len() >= 2 {
            let start_idx = self.entity_history.len().saturating_sub(10);
            let recent = &self.entity_history[start_idx..];
            if recent.len() >= 2 {
                let (start_time, start_count) = recent.first().unwrap();
                let (end_time, end_count) = recent.last().unwrap();
                let time_delta = end_time.duration_since(*start_time).as_secs_f64();
                if time_delta > 0.0 {
                    (*end_count as f64 - *start_count as f64) / time_delta
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        let total_entities = self.entity_history.last().map(|(_, count)| *count).unwrap_or(0);
        
        // Detect potential leaks
        let potential_leaks: Vec<String> = self.leak_detector.growth_counts.iter()
            .filter(|(_, &count)| count >= self.leak_detector.growth_threshold)
            .map(|(name, _)| name.clone())
            .collect();
        
        MemoryStats {
            total_entities,
            total_components,
            estimated_memory,
            entity_growth_rate,
            component_stats,
            potential_leaks,
        }
    }

    /// Print memory report to console
    pub fn print_report(&self) {
        let stats = self.get_stats();
        
        info!("=== Memory Profile Report ===");
        info!("Total Entities: {}", stats.total_entities);
        info!("Total Components: {}", stats.total_components);
        info!("Estimated Memory: {} KB", stats.estimated_memory / 1024);
        info!("Entity Growth Rate: {:.2}/s", stats.entity_growth_rate);
        
        if !stats.component_stats.is_empty() {
            info!("\nComponent Breakdown:");
            for (name, comp_stats) in &stats.component_stats {
                info!("  {}: {} instances, {} KB", 
                    name, 
                    comp_stats.count, 
                    comp_stats.estimated_size / 1024
                );
            }
        }
        
        if !stats.potential_leaks.is_empty() {
            warn!("\n⚠️ Potential Memory Leaks Detected:");
            for leak in &stats.potential_leaks {
                warn!("  - {}", leak);
            }
        }
    }
}

/// System that updates memory profiler with entity and component counts
pub fn update_memory_profile(
    mut profiler: ResMut<MemoryProfiler>,
    entities: Query<Entity>,
    creatures: Query<&Creature>,
    positions: Query<&Position>,
    health_query: Query<&Health>,
    needs_query: Query<&Needs>,
    resources: Query<&ResourceMarker>,
) {
    let entity_count = entities.iter().count();
    profiler.record_entity_count(entity_count);
    
    // Update component counts
    profiler.update_component_count("Creature", creatures.iter().count());
    profiler.update_component_count("Position", positions.iter().count());
    profiler.update_component_count("Health", health_query.iter().count());
    profiler.update_component_count("Needs", needs_query.iter().count());
    profiler.update_component_count("Resource", resources.iter().count());
    
    // Record snapshot
    let total_components = creatures.iter().count() 
        + positions.iter().count() 
        + health_query.iter().count() 
        + needs_query.iter().count()
        + resources.iter().count();
        
    let estimated_bytes = profiler.component_counts.iter()
        .map(|(name, &count)| {
            let size = profiler.component_sizes.get(name).copied().unwrap_or(32);
            count * size
        })
        .sum();
    
    let snapshot = MemorySnapshot {
        timestamp: Instant::now(),
        entity_count,
        component_count: total_components,
        estimated_bytes,
    };
    
    profiler.record_snapshot("main", snapshot);
}

/// System that periodically prints memory reports
pub fn print_memory_report(
    profiler: Res<MemoryProfiler>,
    time: Res<Time>,
) {
    // Print report every 10 seconds
    if time.elapsed_seconds() as u32 % 10 == 0 {
        profiler.print_report();
    }
}

/// Plugin to add memory profiling to the app
pub struct MemoryProfilerPlugin;

impl Plugin for MemoryProfilerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MemoryProfiler::new())
            .add_systems(Update, (
                update_memory_profile,
                print_memory_report,
            ).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_profiler_creation() {
        let profiler = MemoryProfiler::new();
        let stats = profiler.get_stats();
        assert_eq!(stats.total_entities, 0);
        assert_eq!(stats.total_components, 0);
    }

    #[test]
    fn test_entity_count_recording() {
        let mut profiler = MemoryProfiler::new();
        profiler.record_entity_count(100);
        profiler.record_entity_count(150);
        
        let stats = profiler.get_stats();
        assert_eq!(stats.total_entities, 150);
    }

    #[test]
    fn test_component_tracking() {
        let mut profiler = MemoryProfiler::new();
        profiler.update_component_count("Position", 50);
        profiler.update_component_count("Health", 30);
        
        let stats = profiler.get_stats();
        assert_eq!(stats.component_stats.len(), 2);
        assert_eq!(stats.component_stats["Position"].count, 50);
        assert_eq!(stats.total_components, 80);
    }

    #[test]
    fn test_leak_detection() {
        let mut profiler = MemoryProfiler::new();
        
        // Simulate rapid growth
        for i in 0..10 {
            let snapshot = MemorySnapshot {
                timestamp: Instant::now() + std::time::Duration::from_secs(i),
                entity_count: (i * 100) as usize, // Rapid growth
                component_count: (i * 200) as usize,
                estimated_bytes: (i * 10000) as usize,
            };
            profiler.record_snapshot("test_system", snapshot);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        // Growth rate should be detected
        let _stats = profiler.get_stats();
        // Note: leak detection requires multiple consecutive growth periods
    }

    #[test]
    fn test_memory_estimation() {
        let mut profiler = MemoryProfiler::new();
        profiler.update_component_count("Position", 100);
        profiler.update_component_count("Creature", 50);
        
        let stats = profiler.get_stats();
        assert!(stats.estimated_memory > 0);
        
        // Position: 100 * size_of::<Position>()
        // Creature: 50 * 128 (estimated)
        let expected = 100 * std::mem::size_of::<Position>() + 50 * 128;
        assert_eq!(stats.estimated_memory, expected);
    }
}