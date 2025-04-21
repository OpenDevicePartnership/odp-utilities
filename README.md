# ODP Utilities

A collection of Rust utilities focused on embedded systems development. This repository contains standalone crates that can be used independently or together to assist with various aspects of embedded and systems programming.

## Crates

### [bit-register](crates/bit-register/README.md)

A no_std compatible crate for defining and manipulating bit fields in hardware registers. Provides a macro-based approach to create type-safe register definitions with bit field access.

### [debug-non-default](crates/debug-non-default/README.md)

A procedural macro that provides a custom `Debug` implementation which only displays fields that differ from their default values. Particularly useful for configuration structs, large data structures, and debug logs.

## Development

This project uses a workspace structure where each crate can be developed and used independently.

### Building

```bash
# Build all crates
cargo build

# Build a specific crate
cargo build -p bit-register
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p debug-non-default
```

### Documentation

```bash
# Generate and open documentation
cargo doc --open
```

## License

MIT License