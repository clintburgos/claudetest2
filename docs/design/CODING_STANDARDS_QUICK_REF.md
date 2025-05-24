# Coding Standards Quick Reference

## Naming Conventions
```rust
mod snake_case;              // Modules
struct PascalCase;           // Types, Components
fn snake_case() {}          // Functions, methods
const SCREAMING_SNAKE: u32; // Constants
let snake_case = 42;        // Variables
```

## Component Design
```rust
// ✅ GOOD - Small, focused
#[derive(Component, Copy, Clone, Debug)]
struct Position(Vec2);

// ❌ BAD - Too large
struct Everything { /* many fields */ }
```

## System Patterns
```rust
// ✅ GOOD - Specific query with filters
fn system(
    query: Query<&Position, (With<Active>, Changed<Position>)>
) {}

// ✅ GOOD - Parallel iteration
query.par_iter_mut().for_each_mut(|(mut a, b)| {});
```

## Performance Checklist
- [ ] Components < 64 bytes
- [ ] Spatial indexing used
- [ ] LOD implemented
- [ ] Queries filtered
- [ ] Collections pre-allocated

## Error Handling
```rust
// Always use Result for fallible operations
fn load() -> Result<Data, Error> { }

// Custom errors with thiserror
#[derive(Debug, thiserror::Error)]
enum Error { }
```

## Documentation
```rust
/// Brief description.
/// 
/// # Example
/// ```
/// let result = function(param);
/// ```
pub fn function(param: Type) -> Result { }
```

## Git Commits
```
feat(scope): add new feature
fix(scope): fix bug
perf(scope): improve performance
docs(scope): update documentation
```

## Essential Commands
```bash
cargo fmt                    # Format
cargo clippy -- -D warnings  # Lint
cargo test                   # Test
cargo bench                  # Benchmark
cargo flamegraph            # Profile
```

## Common Mistakes
- Large components
- Unfiltered queries  
- String ops in loops
- Missing error handling
- No change detection

---
*Full guides: [CODE_STYLE_GUIDE.md], [BEST_PRACTICES_SUMMARY.md]*