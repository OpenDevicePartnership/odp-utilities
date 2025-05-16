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
- Support for specifying `big_endian` or `little_endian` byte order (defaults to `little_endian`)
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

### Defining a Register with Specific Byte Order

By default, registers are assumed to be little-endian. You can specify `big_endian` if needed.

```rust
use bit_register::bit_register;

bit_register! {
    #[derive(Debug, PartialEq)]
    pub struct BigEndianRegister: big_endian u32 { // Specify big_endian here
        pub field_a: u8 => [0:7],
        pub field_b: u16 => [8:23],
        pub field_c: u8 => [24:31]
    }
}

// Example: We want to represent the conceptual big-endian u32 represented by the bytes [0x78, 0x56, 0x34, 0x12].
// In this conceptual big-endian number:
// - `field_a` (bits 0-7) corresponds to the least significant byte: 0x78.
// - `field_b` (bits 8-23) corresponds to the middle bytes: 0x3456.
// - `field_c` (bits 24-31) corresponds to the most significant byte: 0x12.
let register = BigEndianRegister {
    field_a: 0x78,
    field_b: 0x3456,
    field_c: 0x12,
};

// When converting to a u32 (e.g., for storage or network transmission):
// On a little-endian platform, the big-endian value 0x12345678 is represented as 0x78563412.
let bits: u32 = register.try_into().unwrap();
assert_eq!(bits, u32::from_be_bytes([0x78, 0x56, 0x34, 0x12]));

// To create a register from a u32 value:
// The input `raw_value_from_platform` is a u32 in the platform's native endianness.
// Assume this value was obtained by reading a big-endian data source.
// For instance, if a hardware register or network packet contains the byte sequence [0x12, 0x34, 0x56, 0x78]
// (which is 0x12345678 in big-endian), reading this as a u32 on a little-endian platform
// will result in `raw_value_from_platform` being 0x78563412.
let raw_value_from_platform = u32::from_be_bytes([0x78, 0x56, 0x34, 0x12]);
let register_from_bits = BigEndianRegister::try_from(raw_value_from_platform).unwrap();

// Inside `try_from` for a `big_endian` register, `raw_value_from_platform` (0x[78, 56, 34, 12])
// is byte-swapped to its conceptual big-endian form (0x[12, 34, 56, 78]).
// Fields are then extracted from this conceptual form (0x[12, 34, 56, 78]):
// - field_a (bits 0-7) will be 0x78.
// - field_b (bits 8-23) will be 0x3456.
// - field_c (bits 24-31) will be 0x12.
assert_eq!(register_from_bits.field_a, 0x78);
assert_eq!(register_from_bits.field_b, 0x3456);
assert_eq!(register_from_bits.field_c, 0x12);

bit_register! {
    #[derive(Debug, PartialEq)]
    pub struct LittleEndianRegister: little_endian u16 { // Explicitly little_endian
        pub low_byte: u8 => [0:7],
        pub high_byte: u8 => [8:15]
    }
}

// Contrast this with a little-endian register
let le_register_value = u16::from_le_bytes([0x34, 0x12]);

let le_bits: u16 = LittleEndianRegister {
    low_byte: 0x34,
    high_byte: 0x12,
}
.try_into()
.unwrap();

assert_eq!(le_bits, le_register_value);

let le_reg_from_bits = LittleEndianRegister::try_from(le_register_value).unwrap();
assert_eq!(le_reg_from_bits.low_byte, 0x34);
assert_eq!(le_reg_from_bits.high_byte, 0x12);

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
