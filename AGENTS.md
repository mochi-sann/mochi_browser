# AGENTS.md

This file provides guidelines for coding agents working on this Rust/eframe project.

## Build, Lint, and Test Commands

### Building
```bash
# Native build (debug)
cargo build

# Native build (release)
cargo build --release

# Web build (WASM)
trunk build

# Web dev server (auto-reloads)
trunk serve
```

### Linting and Formatting
```bash
# Check code
cargo check --workspace --all-targets

# Check WASM build
cargo check --workspace --all-features --lib --target wasm32-unknown-unknown

# Format code
cargo fmt

# Format check
cargo fmt --all -- --check

# Run clippy linter
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::all

# Spell check (requires typos-cli)
typos
```

### Testing
```bash
# Run all tests
cargo test --workspace --all-targets --all-features

# Run doc tests
cargo test --workspace --doc

# Run single test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture

# Run specific test file
cargo test --package mochi_browser <test_name>
```

### CI Script
Run all checks (from check.sh):
```bash
cargo check --quiet --workspace --all-targets
cargo check --quiet --workspace --all-features --lib --target wasm32-unknown-unknown
cargo fmt --all -- --check
cargo clippy --quiet --workspace --all-targets --all-features -- -D warnings -W clippy::all
cargo test --quiet --workspace --all-targets --all-features
cargo test --quiet --workspace --doc
trunk build
```

## Code Style Guidelines

### Imports and Modules
- Keep imports at the top, grouped by category
- Use absolute paths within the crate (e.g., `crate::module::item`)
- Platform-specific imports: `#[cfg(not(target_arch = "wasm32"))]`
- Add `use` statements for external crates before local ones

### Formatting
- Run `cargo fmt` before committing
- Follow standard Rust formatting (4 spaces, no tabs)
- Keep line length reasonable (~100 chars preferred)
- Use blank lines between function definitions

### Types
- Prefer concrete types over trait objects when possible
- Use `#[derive(serde::Deserialize, serde::Serialize)]` for structs needing persistence
- Add `#[serde(default)]` to structs with default impls
- Mark non-serializable fields with `#[serde(skip)]`
- Use `Box<dyn std::error::Error>` for generic errors in non-WASM
- Use `Result<T, String>` for simpler error propagation

### Naming Conventions
- Structs: `PascalCase`
- Functions and methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Private fields: `snake_case`
- Use descriptive names over abbreviations

### Platform-Specific Code
- Use `#[cfg(not(target_arch = "wasm32"))]` for native-only code
- Use `#[cfg(target_arch = "wasm32")]` for WASM-only code
- Keep platform differences minimal; abstract when possible
- WASM: Use `wasm-bindgen-futures::spawn_local` for async tasks
- Native: Use `std::thread::spawn` for background work

### Error Handling
- Never ignore errors with `.unwrap()` or `.expect()` unless safe
- Use `?` operator for clean error propagation
- Provide meaningful error messages: `format!("Context: {}", e)`
- Use `Result<T, E>` types for fallible operations
- Log errors appropriately for debugging

### Async and Concurrency
- For native: Use `std::sync::mpsc::channel()` for thread communication
- For WASM: Limit async operations (simpler patterns preferred)
- Spawn background tasks with `std::thread::spawn` (native) or `spawn_local` (WASM)
- Use `request_repaint()` on context to trigger UI updates from async code

### Logging
- Native: Use `env_logger` initialized in main()
- WASM: Use `eframe::WebLogger` initialized in main()
- Prefer `log::debug!`, `log::info!`, `log::warn!`, `log::error!` macros
- Set log level with `RUST_LOG=debug` environment variable

### Lint Configuration
- Project uses extensive linting in Cargo.toml
- `unsafe_code` is denied
- Many clippy lints are enabled and set to warn
- Run clippy before committing
- Fix warnings before pushing

### Dependencies
- Check existing Cargo.toml before adding new crates
- Use feature flags to avoid unnecessary dependencies
- WASM dependencies: Only add to `[target.'cfg(target_arch = "wasm32")'.dependencies]`
- Native dependencies: Add to `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`
- Keep `reqwest` features minimal for WASM compatibility

### Testing
- Write unit tests for pure functions
- Write integration tests for UI components
- Test both native and WASM builds
- Keep tests fast and deterministic
- Use `#[ignore]` for slow tests

### Comments and Documentation
- Add `///` documentation for public APIs
- Keep comments concise and meaningful
- Don't add comments that repeat obvious code
- Document non-trivial algorithm choices
