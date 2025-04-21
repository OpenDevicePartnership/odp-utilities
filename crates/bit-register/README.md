# Bit Register

A no_std compatible crate for defining and manipulating bit fields in hardware registers.

## Overview

This crate provides a macro for defining register types with bit field support, commonly used in embedded systems and hardware interfacing. It allows for type-safe access to bit fields within registers while handling the underlying bit manipulation.

## Features

- Define struct types that map fields to specific bits in a register
- Define enum types with automatic conversion to/from bit representations
- Type-safe access to register bit fields with compile-time checking
- Range validation for field values to prevent overflow
- Support for various integer sizes (u8, u16, u32, u64)
- Support for different field types (boolean, numeric, enum)
- Fully compatible with no_std environments

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
bit-register = "0.1.0"
```

### Defining a Register Struct

```rust
use bit_register::bit_register;

bit_register! {
    #[derive(Debug, PartialEq)]
    pub struct StatusRegister: u16 {
        pub enabled: bool => [0],           // Single bit at position 0
        pub mode: u8 => [1:3],              // 3 bits at positions 1-3
        pub error_code: u8 => [4:7]         // 4 bits at positions 4-7
    }
}

// Create a register instance
let status = StatusRegister {
    enabled: true,
    mode: 2,
    error_code: 5,
};

// Convert to bits
let bits: u16 = status.try_into().unwrap();

// Create from bits
let status_from_bits = StatusRegister::try_from(bits).unwrap();
```

### Defining an Enum with Bit Representation

```rust
use bit_register::bit_register;

bit_register! {
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum OperationMode: u8 {
        Idle = 0,
        Active = 1,
        LowPower = 2,
        Sleep = 3
    }
}

// Use enum in a register definition
bit_register! {
    #[derive(Debug, PartialEq)]
    pub struct ControlRegister: u16 {
        pub enabled: bool => [0],
        pub mode: OperationMode => [1:2],  // 2 bits for mode
        pub priority: u8 => [3:5]          // 3 bits for priority
    }
}
```

## Error Handling

The crate provides error handling for value validation:

```rust
use bit_register::bit_register;

bit_register! {
    pub struct Example: u8 {
        pub value: u8 => [0:3]  // 4 bits can hold values 0-15
    }
}

// Valid value
let valid = Example { value: 15 };
let bits: u8 = valid.try_into().unwrap();  // Ok

// Invalid value (too large for 4 bits)
let invalid = Example { value: 16 };
let result: Result<u8, _> = invalid.try_into();
assert!(result.is_err());  // Error: value exceeds maximum for bit width
```

## Common Use Cases

This crate is particularly useful for:

- Embedded systems programming
- Hardware interface development
- Device drivers
- Memory-mapped registers
- Protocol implementations with bit-level encoding
- Any scenario requiring type-safe bit manipulation

## License

This crate is licensed under the same license as the parent repository. 