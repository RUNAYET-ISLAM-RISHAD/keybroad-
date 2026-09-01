# Core Engine

The core engine is the heart of the Keybroad Bengali keyboard. It handles all typing logic, layout mapping, and text processing.

## Architecture

- **Language:** Rust (for performance and cross-platform portability)
- **Design:** Pure functional — same input always produces same output
- **FFI:** Exports C-compatible functions for JNI (Android) and FFI (iOS)
- **Memory:** Zero-allocation on hot path (arena allocator in production)

## Structure

```
core/
├── src/
│   ├── lib.rs          # Library entry point, re-exports
│   ├── types.rs        # Type definitions (KeyEvent, OutputAction, etc.)
│   └── engine.rs       # BengaliEngine implementation
├── tests/
│   └── engine_tests.rs # Integration tests
├── benches/
│   └── engine_bench.rs # Performance benchmarks
├── layouts/
│   ├── phonetic.json   # Phonetic layout mapping
│   ├── english.json    # English QWERTY layout
│   └── ...             # Other layouts
└── Cargo.toml
```

## Building

```bash
# Build for development
cargo build

# Build for release
cargo build --release

# Build for Android (cross-compilation)
cargo build --target aarch64-linux-android --release

# Build for iOS (cross-compilation)
cargo build --target aarch64-apple-ios --release
```

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_process_key

# Run benchmarks
cargo bench
```

## API

The main API is the `BengaliEngine` struct:

```rust
use keybroad_core::{BengaliEngine, KeyEvent, LayoutType};

// Create engine
let mut engine = BengliEngine::new(LayoutType::Phonetic);

// Process a key
let event = KeyEvent::down(29, 0x0995); // 'k' key
let actions = engine.process_key(event)?;

// Actions tell the platform what to do
for action in actions {
    match action {
        OutputAction::CommitText(text) => { /* display text */ }
        OutputAction::Backspace(n) => { /* delete n chars */ }
        _ => {}
    }
}
```

## Performance Targets

| Operation | Target |
|---|---|
| Single key press | < 2ms |
| 100 rapid key presses | < 20ms |
| Engine creation | < 1ms |
| Memory per keystroke | 0 bytes (arena) |

## Status

- [x] Basic types defined
- [x] Engine skeleton with process_key
- [ ] Trie-based dictionary lookup
- [ ] Bengali conjunct character handling
- [ ] Layout JSON loading
- [ ] N-gram prediction
- [ ] Gesture/swipe typing
- [ ] Arena allocator
