# Performance Quick Reference

## Critical Performance Rules

### 1. **Update Only What's Visible**
```rust
// Always check visibility
if !in_camera_view(entity) { return; }
```

### 2. **Use LOD Everywhere**
- Animations: Full → Simple → Static
- AI: Complex → Basic → Dormant  
- Rendering: Detailed → Simple → Hidden

### 3. **Cache Expensive Calculations**
- Spatial queries: Cache for 0.5-1.0 seconds
- Pathfinding: Store results
- Decision making: Reuse recent decisions

### 4. **Batch Operations**
- Group similar updates
- Process in parallel when possible
- Minimize system switches

## Component Size Guidelines

| Component | Max Size | Reason |
|-----------|----------|--------|
| Position | 8 bytes | Hot path data |
| Velocity | 8 bytes | Updated every frame |
| Health/Needs | 4-8 bytes | Frequently accessed |
| Metadata | No limit | Cold storage, rarely accessed |

## System Update Frequencies

| System | Frequency | LOD Impact |
|--------|-----------|------------|
| Movement | Every frame | Yes - reduce for distant |
| Needs | 10 Hz | No - always update |
| Decisions | 0.5-10 Hz | Yes - 10Hz near, 2Hz medium, 1Hz far, 0.5Hz distant |
| Genetics | On event | No |
| Animations | Every frame | Yes - heavily reduced |

## Memory Budgets

- **Per Creature**: < 1KB hot data
- **World Chunk**: < 64KB
- **Total RAM**: < 2GB target

## Frame Time Budgets (16ms total)

- Input: 1ms
- Simulation: 6ms
- Rendering: 6ms  
- UI: 2ms
- Buffer: 1ms

## Profiling Commands

```bash
# CPU profiling
cargo build --release && perf record -g ./target/release/game
perf report

# Memory profiling  
valgrind --tool=massif ./target/release/game
ms_print massif.out.*

# Tracy real-time profiling
cargo run --release --features tracy
```

## Red Flags 🚩

1. **Components > 64 bytes**
2. **Allocations in hot loops**
3. **Uncached spatial queries**
4. **No LOD on distant entities**
5. **String operations in systems**
6. **HashMap in hot paths**
7. **Unbounded growth**
8. **No frustum culling**

## Quick Wins 🎯

1. **Enable LTO**: 10-20% performance
2. **Use par_iter_mut**: Near-linear scaling
3. **Frustum culling**: 50%+ fewer entities
4. **LOD system**: 70%+ reduction in work
5. **Spatial indexing**: O(n²) → O(n log n)

---
*Full guide: [PERFORMANCE_OPTIMIZATION.md](./PERFORMANCE_OPTIMIZATION.md)*