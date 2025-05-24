# Best Practices Summary

## Code Organization

### Project Structure
```
src/
├── main.rs           # App setup only
├── lib.rs            # Public API
├── simulation/       # Core logic (creatures, genetics, needs)
├── world/           # World generation and management
├── rendering/       # Visual systems
├── ui/              # User interface
└── utils/           # Shared utilities
```

### Module Guidelines
- One concept per module
- Public API at top of file
- Private implementation below
- Re-export commonly used items

## Bevy Best Practices

### Components
```rust
// ✅ GOOD: Small, focused components
#[derive(Component, Debug, Clone, Copy)]
struct Position(Vec2);

#[derive(Component, Debug, Clone, Copy)]
struct Velocity(Vec2);

// ❌ BAD: Monolithic components
struct CreatureData { /* 20+ fields */ }
```

### Systems
```rust
// ✅ GOOD: Specific queries with filters
fn update_hungry_creatures(
    mut query: Query<(&mut Needs, &Position), (With<Hungry>, Without<Sleeping>)>,
) { }

// ✅ GOOD: Use change detection
fn react_to_damage(
    query: Query<&Health, Changed<Health>>,
) { }

// ❌ BAD: Overly broad queries
fn update_all_transforms(
    mut query: Query<&mut Transform>, // Gets EVERYTHING!
) { }
```

### Plugins
- Group related functionality
- One plugin per major system
- Clear dependencies between plugins

## Performance Rules

### Critical Performance Guidelines
1. **Components < 64 bytes** for cache efficiency
2. **Use spatial indexing** for all proximity queries
3. **Implement LOD** for animations and AI
4. **Profile before optimizing** but design for performance

### Query Optimization
```rust
// Use filters aggressively
Query<&Position, (With<Visible>, Without<Sleeping>)>

// Pre-allocate collections
let mut results = Vec::with_capacity(expected_size);

// Early returns
if !entity.is_active() { return; }
```

## Error Handling

### Use Result Types
```rust
pub fn load_data(path: &Path) -> Result<Data, DataError> {
    // Explicit error handling
}

// Custom error types with thiserror
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Testing

### Test Organization
- Unit tests in same file (`#[cfg(test)]`)
- Integration tests in `tests/` directory
- Benchmarks in `benches/` directory

### Test Patterns
```rust
// Bevy system testing
let mut app = App::new();
app.add_plugins(MinimalPlugins);
app.add_systems(Update, system_to_test);
app.update();
```

## Documentation

### Module Docs
```rust
//! # Module Name
//! 
//! Brief description of module purpose.
//! 
//! ## Example
//! ```
//! use module::function;
//! ```
```

### Function Docs
```rust
/// Brief description.
/// 
/// # Arguments
/// * `param` - Description
/// 
/// # Returns
/// Description of return value
pub fn function(param: Type) -> ReturnType { }
```

## Git Workflow

### Branch Naming
- `feat/description` - New features
- `fix/description` - Bug fixes
- `perf/description` - Performance improvements
- `docs/description` - Documentation

### Commit Format
```
type(scope): subject

Longer description if needed.

Closes #123
```

## Development Tools

### Essential Commands
```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run tests
cargo test

# Watch for changes
cargo watch -x run

# Profile performance
cargo flamegraph
```

### VS Code Settings
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "editor.formatOnSave": true
}
```

## Common Pitfalls to Avoid

### Bevy-Specific
- Forgetting `.chain()` for system ordering
- Modifying entities without `Commands`
- Broad queries without filters
- Not using change detection

### Performance
- Large components (> 64 bytes)
- String allocations in hot loops
- Missing spatial indexing
- No LOD for distant entities

### Rust-Specific
- Cloning when borrowing suffices
- Not handling all error cases
- Overuse of `RefCell`/`Mutex`
- Missing `#[inline]` on small functions

## Quick Checklist

Before committing:
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` has no warnings
- [ ] Tests pass (`cargo test`)
- [ ] Documentation updated
- [ ] Performance impact considered
- [ ] Error cases handled

---
*For detailed guides see:*
- [CODE_STYLE_GUIDE.md](./CODE_STYLE_GUIDE.md) - Complete style guide
- [DEVELOPMENT_WORKFLOW.md](./DEVELOPMENT_WORKFLOW.md) - Development process
- [PERFORMANCE_OPTIMIZATION.md](./PERFORMANCE_OPTIMIZATION.md) - Performance guide