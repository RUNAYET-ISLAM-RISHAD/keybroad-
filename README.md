# Keybroad — Modern Bengali Keyboard

A privacy-first, AI-powered Bengali keyboard with zero-latency input and on-device intelligence.

## Architecture

```
keybroad/
├── core/          # Rust core engine (cross-platform via JNI/FFI)
├── android/       # Android app (Jetpack Compose)
├── ios/           # iOS app (SwiftUI)
├── server/        # Backend (Node.js + Fastify)
├── data/          # Dictionaries, n-grams, golden test files
└── scripts/       # Build and utility scripts
```

## Core Engine

The core engine is written in Rust for maximum performance and cross-platform portability. It compiles to:
- `libkeybroad_core.so` for Android (via JNI)
- `libkeybroad_core.a` for iOS (via FFI)
- `keybroad_core.wasm` for future WebAssembly support

### Building

```bash
# Build for host (development/testing)
cd core
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Development

See `ARCHITECTURE.md` for the complete technical design document.

## License

Proprietary — All rights reserved.
