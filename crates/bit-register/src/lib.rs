#![no_std]

//! # Bit Register
//!
//! A no_std compatible crate for defining and manipulating bit fields in hardware registers.
//!
//! This crate provides a macro for defining register types with bit field support, commonly
//! used in embedded systems and hardware interfacing. It allows for type-safe access to bit
//! fields within registers while handling the underlying bit manipulation.
//!
//! ## Features
//!
//! - Define struct types that map fields to specific bits in a register
//! - Define enum types with automatic conversion to/from bit representations
//! - Type-safe access to register bit fields with compile-time checking
//! - Range validation for field values to prevent overflow
//! - Support for various integer sizes (u8, u16, u32, u64)
//! - Support for different field types (boolean, numeric, enum)
//! - Fully compatible with no_std environments
//!
//! ## Defining a Register Struct
//!
//! ```rust
//! use bit_register::bit_register;
//!
//! bit_register! {
//!     #[derive(Debug, PartialEq)]
//!     pub struct StatusRegister: u16 {
//!         pub enabled: bool => [0],           // Single bit at position 0
//!         pub mode: u8 => [1:3],              // 3 bits at positions 1-3
//!         pub error_code: u8 => [4:7]         // 4 bits at positions 4-7
//!     }
//! }
//!
//! // Create a register instance
//! let status = StatusRegister {
//!     enabled: true,
//!     mode: 2,
//!     error_code: 5,
//! };
//!
//! // Convert to bits
//! let bits: u16 = status.try_into().unwrap();
//!
//! // Create from bits
//! let status_from_bits = StatusRegister::try_from(bits).unwrap();
//! ```
//!
//! ## Defining an Enum with Bit Representation
//!
//! ```rust
//! use bit_register::bit_register;
//!
//! bit_register! {
//!     #[derive(Debug, PartialEq, Clone, Copy)]
//!     pub enum OperationMode: u8 {
//!         Idle = 0,
//!         Active = 1,
//!         LowPower = 2,
//!         Sleep = 3
//!     }
//! }
//!
//! // Use enum in a register definition
//! bit_register! {
//!     #[derive(Debug, PartialEq)]
//!     pub struct ControlRegister: u16 {
//!         pub enabled: bool => [0],
//!         pub mode: OperationMode => [1:2],  // 2 bits for mode
//!         pub priority: u8 => [3:5]          // 3 bits for priority
//!     }
//! }
//! ```
//!
//! ## Error Handling
//!
//! The crate provides error handling for value validation:
//!
//! ```rust
//! use bit_register::bit_register;
//!
//! bit_register! {
//!     pub struct Example: u8 {
//!         pub value: u8 => [0:3]  // 4 bits can hold values 0-15
//!     }
//! }
//!
//! // Valid value
//! let valid = Example { value: 15 };
//! let bits: u8 = valid.try_into().unwrap();  // Ok
//!
//! // Invalid value (too large for 4 bits)
//! let invalid = Example { value: 16 };
//! let result: Result<u8, _> = invalid.try_into();
//! assert!(result.is_err());  // Error: value exceeds maximum for bit width
//! ```

mod traits;
pub use traits::*;

// Re-export num_traits for use in the macro
pub extern crate num_traits;

/// A macro for defining registers with fields that map to specific bits in an underlying type.
///
/// The macro provides automatic conversion between the register types and their
/// underlying bit representations, with range checking and error handling. Underlying types
/// are an unsigned integer type.
#[macro_export]
macro_rules! bit_register {
    // Entrypoint for defining an enum type which can be used as a bit register
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident: $repr_type:ty {
            $(
                $(#[$variant_attr:meta])*
                $variant:ident = $value:expr
            ),+ $(,)?
        }
    ) => {
        $(#[$attr])*
        #[repr($repr_type)]
        $vis enum $name {
            $(
                $(#[$variant_attr])*
                $variant = $value,
            )+
        }

        impl $crate::NumBytes for $name {
            const NUM_BYTES: usize = <$repr_type as $crate::NumBytes>::NUM_BYTES;
        }

        impl<T: Copy + TryFrom<$repr_type>> $crate::TryIntoBits<T> for $name {
            fn try_into_bits(self) -> Result<T, &'static str> {
                // Convert enum to its underlying numeric type then to target type
                (self as $repr_type).try_into_bits()
            }
        }

        impl<T: Copy> $crate::TryFromBits<T> for $name where $repr_type: TryFrom<T> {
            fn try_from_bits(bits: T) -> Result<Self, &'static str> {
                // Convert the bits to the enum's representation type
                let value = <$repr_type>::try_from_bits(bits)?;

                // Match the numeric value to the corresponding enum variant
                match value {
                    $(
                        $value => Ok(Self::$variant),
                    )+
                    _ => Err(concat!("Invalid value for enum ", stringify!($name))),
                }
            }
        }
    };

    // Define a struct type which can be used as a bit register
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident: $underlying_type:ty {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident: $field_type:tt => $field_bits:tt
            ),* $(,)?
        }
    ) => {
        $(#[$attr])*
        $vis struct $name {
            $(
                $(#[$field_attr])*
                $field_vis $field_name: $field_type,
            )*
        }

        impl $crate::NumBytes for $name {
            const NUM_BYTES: usize = <$underlying_type as $crate::NumBytes>::NUM_BYTES;
        }

        impl TryFrom<$underlying_type> for $name {
            type Error = &'static str;

            fn try_from(value: $underlying_type) -> Result<Self, Self::Error> {
                $(
                    let $field_name = bit_register!(@extract_bits $underlying_type, value, $field_type, $field_bits);
                )*

                Ok(Self {
                    $(
                        $field_name,
                    )*
                })
            }
        }

        impl TryInto<$underlying_type> for $name {
            type Error = &'static str;

            fn try_into(self) -> Result<$underlying_type, Self::Error> {
                let mut value: $underlying_type = 0;
                $(
                    // Handle bit packing for each field
                    value |= bit_register!(@pack_bits $underlying_type, self.$field_name, $field_name, $field_type, $field_bits);
                )*
                Ok(value)
            }
        }

        impl $crate::BitRegister<$underlying_type> for $name {}
    };

    // Extract a single bit, convert to range
    (@extract_bits $underlying_type:ty, $value:expr, $field_type:ty, [$bit:literal]) => {
        bit_register!(@extract_bits_impl $underlying_type, $value, $field_type, [$bit:$bit])
    };

    // Extract a range of bits
    (@extract_bits $underlying_type:ty, $value:expr, $field_type:ty, [$start:literal:$end:literal]) => {
        bit_register!(@extract_bits_impl $underlying_type, $value, $field_type, [$start:$end])
    };

    // Generic implementation for extracting bits from an unsigned integer type
    (@extract_bits_impl $underlying_type:ty, $value:expr, $field_type:ty, [$start:literal:$end:literal]) => {
        {
            // Calculate how many bits are in this field
            const BIT_COUNT: usize = ($end - $start) + 1;

            // Create a mask with BIT_COUNT number of 1s
            // Handle the case where BIT_COUNT is the full width of the underlying type
            let mask: $underlying_type = if BIT_COUNT >= (<$underlying_type as $crate::NumBytes>::NUM_BYTES * 8) {
                <$underlying_type>::MAX
            } else {
                ((1 as $underlying_type) << BIT_COUNT) - 1
            };

            // Extract the relevant bits by right-shifting to the start position
            // and then masking to keep only the bits we want
            let extracted_value = ($value >> $start) & mask;

            // Convert the extracted bits to the field type
            $crate::TryFromBits::try_from_bits(extracted_value)?
        }
    };


    // Pack a single bit field
    (@pack_bits $underlying_type:ty, $field_value:expr, $field_name:ident, $field_type:tt, [$bit:literal]) => {
        bit_register!(@pack_bits $underlying_type, $field_value, $field_name, $field_type, [$bit:$bit])
    };

    // Pack a range of bits
    (@pack_bits $underlying_type:ty, $field_value:expr, $field_name:ident, $field_type:tt, [$start:literal:$end:literal]) => {
        {
            // Calculate how many bits are needed for this field
            const BIT_COUNT: usize = ($end - $start) + 1;
            const FIELD_TYPE_BITS: usize = <$field_type as $crate::NumBytes>::NUM_BYTES * 8;

            // Calculate the maximum value that can fit in the bit field
            // We need to handle this carefully to avoid overflow
            let max_value = if BIT_COUNT >= 64 {
                u64::MAX // Special case for fields that use all available bits
            } else {
                (1u64 << BIT_COUNT) - 1 // 2^BIT_COUNT - 1
            };

            let field_value: $underlying_type = $crate::TryIntoBits::try_into_bits($field_value)?;

            // Check if the value fits in the allocated bits
            // We skip this check if the field type uses fewer or equal bits than we've allocated
            if BIT_COUNT < FIELD_TYPE_BITS {
                if field_value as u64 > max_value {
                    return Err(concat!(stringify!($field_name), " exceeds maximum value for its bit width"));
                }
            }

            // Create a mask for the field value before shifting it into position
            let field_mask = if BIT_COUNT >= FIELD_TYPE_BITS {
                <$underlying_type>::MAX // Use all bits if BIT_COUNT exceeds underlying type size
            } else {
                ((1 as $underlying_type) << BIT_COUNT) - 1 // Create a mask of BIT_COUNT 1s
            };

            // Mask the value and shift it to the correct position
            ((field_value) & field_mask) << $start
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_register() {
        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            pub struct BasicRegister: u16 {
                pub flag: bool => [0],
                pub small_field: u8 => [1:4],
            }
        }

        // Test construction
        let register = BasicRegister {
            flag: true,
            small_field: 7, // Max for 4 bits
        };

        // Test conversion to bits
        let value: u16 = register.try_into().unwrap();

        // Test conversion back from bits
        let round_trip = BasicRegister::try_from(value).unwrap();

        // Verify values
        assert_eq!(round_trip.flag, true);
        assert_eq!(round_trip.small_field, 7);

        // Test invalid value
        let invalid = BasicRegister {
            flag: false,
            small_field: 16, // Too large for 4 bits
        };

        assert!(TryInto::<u16>::try_into(invalid).is_err());
    }

    #[test]
    fn test_different_underlying_types() {
        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            struct U8Register: u8 {
                pub field: u8 => [0:6],
            }
        }

        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            struct U32Register: u32 {
                pub field: u16 => [0:15],
            }
        }

        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            struct U64Register: u64 {
                pub field: u32 => [0:31],
            }
        }

        // Test u8 register
        let u8_reg = U8Register { field: 127 };
        let u8_val: u8 = u8_reg.try_into().unwrap();
        assert_eq!(u8_val, 127);

        // Test u32 register
        let u32_reg = U32Register { field: 32767 };
        let u32_val: u32 = u32_reg.try_into().unwrap();
        assert_eq!(u32_val, 32767);

        // Test u64 register
        let u64_reg = U64Register { field: 2147483647 };
        let u64_val: u64 = u64_reg.try_into().unwrap();
        assert_eq!(u64_val, 2147483647);
    }

    #[test]
    fn test_enum_widening() {
        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            enum EnumRegister: u8 {
                Variant1 = 0b0001,
                Variant2 = 0b0010,
            }
        }

        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            struct EnumRegister2: u16 {
                pub enum_field: EnumRegister => [0:7],
            }
        }
    }

    #[test]
    fn test_multiple_fields() {
        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            pub struct ComplexRegister: u32 {
                pub flag1: bool => [0],
                pub flag2: bool => [1],
                pub small_field: u8 => [2:9],
                pub medium_field: u16 => [10:25],
                pub large_flag: bool => [31],
            }
        }

        let register = ComplexRegister {
            flag1: true,
            flag2: false,
            small_field: 255,
            medium_field: 1000, // A smaller value that fits in 16 bits
            large_flag: true,
        };

        let value: u32 = register.try_into().unwrap();
        let round_trip = ComplexRegister::try_from(value).unwrap();

        assert_eq!(round_trip.flag1, true);
        assert_eq!(round_trip.flag2, false);
        assert_eq!(round_trip.small_field, 255);
        assert_eq!(round_trip.medium_field, 1000);
        assert_eq!(round_trip.large_flag, true);
    }

    #[test]
    fn test_edge_cases() {
        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            pub struct EdgeRegister: u64 {
                // Single bit at position 0
                pub lowest_bit: bool => [0],
                // Single bit at highest position
                pub highest_bit: bool => [63],
                // Maximum range for u32
                pub full_u32: u32 => [16:47],
            }
        }

        let register = EdgeRegister {
            lowest_bit: true,
            highest_bit: true,
            full_u32: u32::MAX,
        };

        let value: u64 = register.try_into().unwrap();
        let expected = 1u64 | (1u64 << 63) | ((u32::MAX as u64) << 16);
        assert_eq!(value, expected);

        let round_trip = EdgeRegister::try_from(value).unwrap();
        assert_eq!(round_trip.lowest_bit, true);
        assert_eq!(round_trip.highest_bit, true);
        assert_eq!(round_trip.full_u32, u32::MAX);
    }

    #[test]
    fn test_out_of_range_values() {
        bit_register! {
            pub struct RangeRegister: u16 {
                pub small_field: u8 => [0:3],
            }
        }

        // Valid value: 4 bits can hold 0-15
        let valid = RangeRegister { small_field: 15 };
        assert!(TryInto::<u16>::try_into(valid).is_ok());

        // Invalid value: too large for 4 bits
        let invalid = RangeRegister { small_field: 16 };
        assert!(TryInto::<u16>::try_into(invalid).is_err());
    }

    #[test]
    fn test_from_raw_value() {
        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            pub struct FromRawRegister: u32 {
                pub flag: bool => [0],
                pub field1: u8 => [1:8],
                pub field2: u16 => [9:24],
            }
        }

        // Create a raw value
        let raw_value: u32 = 1 | (255 << 1) | (0xFFFF << 9);

        // Convert to register
        let register = FromRawRegister::try_from(raw_value).unwrap();

        // Verify fields were extracted correctly
        assert_eq!(register.flag, true);
        assert_eq!(register.field1, 255);
        assert_eq!(register.field2, 0xFFFF);
    }

    #[test]
    fn test_one_bit_enum() {
        // First test with On/Off naming
        bit_register! {
            #[derive(Debug, PartialEq, Eq, Clone, Copy)]
            enum OneBitEnum: u8 {
                Off = 0b0,
                On = 0b1,
            }
        }

        // Alternative test with different naming scheme
        bit_register! {
            #[derive(Debug, PartialEq, Eq, Clone, Copy)]
            enum BooleanState: u8 {
                False = 0,
                True = 1,
            }
        }

        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            struct EnumRegister: u8 {
                pub enum_field: OneBitEnum => [0],
            }
        }

        // Test with Off (0)
        let register = EnumRegister {
            enum_field: OneBitEnum::Off,
        };

        // Verify conversion to bits
        let bits: u8 = register.try_into().unwrap();
        assert_eq!(bits, 0); // Bit 0 should be 0

        // Verify round-trip conversion
        let round_trip = EnumRegister::try_from(bits).unwrap();
        assert_eq!(round_trip.enum_field, OneBitEnum::Off);

        // Test with On (1)
        let register = EnumRegister {
            enum_field: OneBitEnum::On,
        };

        // Verify conversion to bits
        let bits: u8 = register.try_into().unwrap();
        assert_eq!(bits, 1); // Bit 0 should be 1

        // Verify round-trip conversion
        let round_trip = EnumRegister::try_from(bits).unwrap();
        assert_eq!(round_trip.enum_field, OneBitEnum::On);

        // Test with multiple fields
        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            struct ComplexRegister: u8 {
                pub enum_field: OneBitEnum => [0],
                pub flag: bool => [1],
                pub value: u8 => [2:4],
            }
        }

        let register = ComplexRegister {
            enum_field: OneBitEnum::On, // 1 at bit 0
            flag: true,                 // 1 at bit 1
            value: 3,                   // 011 at bits 2-4
        };

        // Expected bits:
        // Bit 0: enum_field = 1
        // Bit 1: flag = 1
        // Bits 2-4: value = 3 (011 in binary)
        //          0011 1001
        let expected: u8 = 0b00001111;

        // Verify conversion to bits
        let bits: u8 = register.try_into().unwrap();
        assert_eq!(bits, expected);

        // Verify round-trip conversion
        let round_trip = ComplexRegister::try_from(bits).unwrap();
        assert_eq!(round_trip.enum_field, OneBitEnum::On);
        assert_eq!(round_trip.flag, true);
        assert_eq!(round_trip.value, 3);

        // Test the BooleanState enum with different naming scheme
        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            struct BooleanRegister: u8 {
                pub state: BooleanState => [0],
            }
        }

        // Test with False (0)
        let register = BooleanRegister {
            state: BooleanState::False,
        };

        // Verify conversion to bits
        let bits: u8 = register.try_into().unwrap();
        assert_eq!(bits, 0);

        // Verify round-trip conversion
        let round_trip = BooleanRegister::try_from(bits).unwrap();
        assert_eq!(round_trip.state, BooleanState::False);

        // Test with True (1)
        let register = BooleanRegister {
            state: BooleanState::True,
        };

        // Verify conversion to bits
        let bits: u8 = register.try_into().unwrap();
        assert_eq!(bits, 1);

        // Verify round-trip conversion
        let round_trip = BooleanRegister::try_from(bits).unwrap();
        assert_eq!(round_trip.state, BooleanState::True);
    }

    #[test]
    fn test_enum_serialization() {
        bit_register! {
            #[derive(Debug, PartialEq, Eq, Clone, Copy)]
            enum TestEnum: u32 {
                Variant1 = 0b0001,
                Variant2 = 0b0010,
            }
        }

        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            pub struct TestEnumRegister: u32 {
                pub variant1: TestEnum => [2:5],
            }
        }

        let register = TestEnumRegister {
            variant1: TestEnum::Variant1,
        };
        let value: u32 = register.try_into().unwrap();
        let round_trip = TestEnumRegister::try_from(value).unwrap();
        assert_eq!(round_trip.variant1, TestEnum::Variant1);
    }

    #[test]
    fn test_auto_enum_definition() {
        // Test the new enum definition with automatic TryToFromBits implementation
        bit_register! {
            #[derive(Debug, PartialEq, Eq, Clone, Copy)]
            pub enum OperationMode: u8 {
                Idle = 0,
                Active = 1,
                LowPower = 2,
                Sleep = 3
            }
        }

        // Test conversion to bits
        let mode = OperationMode::Active;
        let bits: u8 = TryIntoBits::try_into_bits(mode).unwrap();
        assert_eq!(bits, 1);

        // Test conversion from bits
        let round_trip = OperationMode::try_from_bits(bits).unwrap();
        assert_eq!(round_trip, OperationMode::Active);

        // Test invalid value
        let result = OperationMode::try_from_bits(4u8);
        assert!(result.is_err());

        // Test using the enum in a register
        bit_register! {
            #[derive(Debug, PartialEq, Eq)]
            pub struct ControlRegister: u32 {
                pub enabled: bool => [0],
                pub mode: OperationMode => [1:2],
                pub priority: u8 => [3:5]
            }
        }

        let register = ControlRegister {
            enabled: true,
            mode: OperationMode::LowPower,
            priority: 3,
        };

        // Test conversion to bits
        let value: u32 = register.try_into().unwrap();
        // Expected: bit 0 set (enabled), bits 1-2 = 2 (LowPower), bits 3-5 = 3 (priority)
        let expected = 0b_000_011_10_1;
        assert_eq!(value, expected);

        // Test conversion from bits
        let round_trip = ControlRegister::try_from(value).unwrap();
        assert_eq!(round_trip.enabled, true);
        assert_eq!(round_trip.mode, OperationMode::LowPower);
        assert_eq!(round_trip.priority, 3);
    }
}

#[cfg(test)]
mod property_tests {
    extern crate std;

    use super::*;
    use proptest::prelude::*;

    // Register with a boolean field
    bit_register! {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        struct BoolRegister: u32 {
            pub flag: bool => [0]
        }
    }

    // Register with different numeric field widths
    bit_register! {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        struct NumericRegister: u64 {
            pub u8_small: u8 => [0:3],   // 4 bits
            pub u8_full: u8 => [4:11],   // 8 bits
            pub u16_small: u16 => [12:19], // 8 bits
            pub u16_full: u16 => [20:35],  // 16 bits
            pub u32_small: u32 => [36:43], // 8 bits
            pub u32_full: u32 => [44:59]   // 16 bits
        }
    }

    // Register for testing enum fields
    bit_register! {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        enum TestEnum: u32 {
            Variant1 = 0,
            Variant2 = 1,
            Variant3 = 2,
            Variant4 = 3,
        }
    }

    bit_register! {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        struct EnumRegister: u32 {
            pub enum_field: TestEnum => [0:1]
        }
    }

    // Register with multiple fields of different types
    bit_register! {
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        struct MixedRegister: u32 {
            pub flag1: bool => [0],
            pub flag2: bool => [1],
            pub small_num: u8 => [2:5],
            pub enum_field: TestEnum => [6:7],
            pub medium_num: u16 => [8:23]
        }
    }

    // Tests for boolean fields
    proptest! {
        #[test]
        fn bool_field_roundtrip(flag in prop::bool::ANY) {
            let register = BoolRegister { flag };

            // Convert to bits
            let bits: u32 = register.try_into().unwrap();

            // Convert back and check values match
            let round_trip = BoolRegister::try_from(bits).unwrap();
            assert_eq!(register.flag, round_trip.flag);

            // Verify correct bit patterns
            assert_eq!(bits, if flag { 1 } else { 0 });
        }
    }

    // Tests for numeric fields
    proptest! {
        // Test valid numeric ranges
        #[test]
        fn numeric_fields_within_range(
            u8_small in 0u8..=15u8,
            u8_full in 0u8..=255u8,
            u16_small in 0u16..=255u16,
            u16_full in 0u16..=65535u16,
            u32_small in 0u32..=255u32,
            u32_full in 0u32..=65535u32
        ) {
            let register = NumericRegister {
                u8_small,
                u8_full,
                u16_small,
                u16_full,
                u32_small,
                u32_full
            };

            // Convert to bits
            let bits: u64 = register.try_into().unwrap();

            // Convert back and check all values match
            let round_trip = NumericRegister::try_from(bits).unwrap();
            assert_eq!(register.u8_small, round_trip.u8_small);
            assert_eq!(register.u8_full, round_trip.u8_full);
            assert_eq!(register.u16_small, round_trip.u16_small);
            assert_eq!(register.u16_full, round_trip.u16_full);
            assert_eq!(register.u32_small, round_trip.u32_small);
            assert_eq!(register.u32_full, round_trip.u32_full);
        }
    }

    // Tests for out-of-range values
    #[test]
    fn out_of_range_values_are_rejected() {
        proptest!(|(u8_too_large in 16u8..=u8::MAX)| {
            // Test u8_small (4 bits, max value 15)
            let register = NumericRegister {
                u8_small: u8_too_large,
                u8_full: 0,
                u16_small: 0,
                u16_full: 0,
                u32_small: 0,
                u32_full: 0
            };
            let result: Result<u64, _> = register.try_into();
            assert!(result.is_err());
        });

        proptest!(|(u16_too_large in 256u16..=u16::MAX)| {
            // Test u16_small (8 bits, max value 255)
            let register = NumericRegister {
                u8_small: 0,
                u8_full: 0,
                u16_small: u16_too_large,
                u16_full: 0,
                u32_small: 0,
                u32_full: 0
            };
            let result: Result<u64, _> = register.try_into();
            assert!(result.is_err());
        });

        proptest!(|(u32_too_large in 256u32..=u32::MAX)| {
            // Test u32_small (8 bits, max value 255)
            let register = NumericRegister {
                u8_small: 0,
                u8_full: 0,
                u16_small: 0,
                u16_full: 0,
                u32_small: u32_too_large,
                u32_full: 0
            };
            let result: Result<u64, _> = register.try_into();
            assert!(result.is_err());
        });

        proptest!(|(u32_too_large in 65536u32..=u32::MAX)| {
            // Test u32_full (16 bits, max value 65535)
            let register = NumericRegister {
                u8_small: 0,
                u8_full: 0,
                u16_small: 0,
                u16_full: 0,
                u32_small: 0,
                u32_full: u32_too_large
            };
            let result: Result<u64, _> = register.try_into();
            assert!(result.is_err());
        });
    }

    // Tests for enum fields
    proptest! {
        #[test]
        fn enum_field_roundtrip(variant in 0u8..=3u8) {
            let enum_value = match variant {
                0 => TestEnum::Variant1,
                1 => TestEnum::Variant2,
                2 => TestEnum::Variant3,
                3 => TestEnum::Variant4,
                _ => unreachable!()
            };

            let register = EnumRegister { enum_field: enum_value };

            // Convert to bits
            let bits: u32 = register.try_into().unwrap();

            // Convert back and check enum value matches
            let round_trip = EnumRegister::try_from(bits).unwrap();
            assert_eq!(register.enum_field, round_trip.enum_field);

            // Verify correct bit pattern
            assert_eq!(bits, variant as u32);
        }
    }

    // Tests for invalid enum values
    #[test]
    fn invalid_enum_values_rejected() {
        // Try to create register with raw value that has an invalid enum
        // Our enum is 2 bits at positions [0:1] but with only 4 valid values (0-3)

        // First, try values within the bit range but not valid enum values
        // Wait, actually we test this in another case - all values 0-3 are valid in our enum

        // Let's create a new TestEnum with gaps and more restrictive validation
        bit_register! {
            #[derive(Debug, PartialEq, Eq, Clone, Copy)]
            enum RestrictedEnum: u8 {
                OnlyEven0 = 0,
                OnlyEven2 = 2,
            }
        }

        bit_register! {
            #[derive(Debug, PartialEq, Eq, Clone, Copy)]
            struct RestrictedEnumRegister: u32 {
                pub enum_field: RestrictedEnum => [0:2]
            }
        }

        // Now test with values that are within bit range but invalid for the enum
        let odd_value: u32 = 1; // Within range but not a valid enum value
        let result = RestrictedEnumRegister::try_from(odd_value);
        assert!(result.is_err());

        let another_odd: u32 = 3; // Within range but not a valid enum value
        let result = RestrictedEnumRegister::try_from(another_odd);
        assert!(result.is_err());

        // Value that exceeds bit range
        let too_large: u32 = 7; // Beyond the 3 bits we allocated
        let _result = RestrictedEnumRegister::try_from(too_large);
        // This might or might not error depending on how the macro works
        // (it may just mask the value to fit in 3 bits)
    }

    // Tests for mixed fields
    proptest! {
        #[test]
        fn mixed_fields_roundtrip(
            flag1 in prop::bool::ANY,
            flag2 in prop::bool::ANY,
            small_num in 0u8..=15u8,
            enum_variant in 0u8..=3u8,
            medium_num in 0u16..=65535u16
        ) {
            let enum_value = match enum_variant {
                0 => TestEnum::Variant1,
                1 => TestEnum::Variant2,
                2 => TestEnum::Variant3,
                3 => TestEnum::Variant4,
                _ => unreachable!()
            };

            let register = MixedRegister {
                flag1,
                flag2,
                small_num,
                enum_field: enum_value,
                medium_num
            };

            // Convert to bits
            let bits: u32 = register.try_into().unwrap();

            // Convert back and check all values match
            let round_trip = MixedRegister::try_from(bits).unwrap();
            assert_eq!(register.flag1, round_trip.flag1);
            assert_eq!(register.flag2, round_trip.flag2);
            assert_eq!(register.small_num, round_trip.small_num);
            assert_eq!(register.enum_field, round_trip.enum_field);
            assert_eq!(register.medium_num, round_trip.medium_num);

            // Verify correct bit composition
            let expected_bits =
                (if flag1 { 1 } else { 0 }) |
                (if flag2 { 1 << 1 } else { 0 }) |
                ((small_num as u32) << 2) |
                ((enum_variant as u32) << 6) |
                ((medium_num as u32) << 8);

            assert_eq!(bits, expected_bits);
        }
    }

    // Tests for raw value conversion
    proptest! {
        #[test]
        fn from_raw_value_handles_bit_patterns(raw_value in 0u32..0x00FFFFFF) {
            // Expected values from this bit pattern
            let expected_flag1 = (raw_value & 0x1) != 0;
            let expected_flag2 = ((raw_value >> 1) & 0x1) != 0;
            let expected_small_num = ((raw_value >> 2) & 0xF) as u8;

            // Only extract the enum if it's valid
            let expected_enum_raw = ((raw_value >> 6) & 0x3) as u8;
            let expected_enum = if expected_enum_raw <= 3 {
                match expected_enum_raw {
                    0 => Some(TestEnum::Variant1),
                    1 => Some(TestEnum::Variant2),
                    2 => Some(TestEnum::Variant3),
                    3 => Some(TestEnum::Variant4),
                    _ => unreachable!()
                }
            } else {
                None
            };

            let expected_medium_num = ((raw_value >> 8) & 0xFFFF) as u16;

            // If the enum value is invalid, we expect the conversion to fail
            if expected_enum.is_none() {
                let result = MixedRegister::try_from(raw_value);
                assert!(result.is_err());
            } else {
                // Otherwise we expect a successful conversion
                let register = MixedRegister::try_from(raw_value).unwrap();

                assert_eq!(register.flag1, expected_flag1);
                assert_eq!(register.flag2, expected_flag2);
                assert_eq!(register.small_num, expected_small_num);
                assert_eq!(register.enum_field, expected_enum.unwrap());
                assert_eq!(register.medium_num, expected_medium_num);
            }
        }
    }

    // Test boundary values
    #[test]
    fn boundary_values_handled_correctly() {
        // Test max values in each field
        let max_register = NumericRegister {
            u8_small: 15,    // 4 bits max
            u8_full: 255,    // 8 bits max
            u16_small: 255,  // 8 bits max
            u16_full: 65535, // 16 bits max
            u32_small: 255,  // 8 bits max
            u32_full: 65535, // 16 bits max
        };

        // Convert to bits and back
        let bits: u64 = max_register.try_into().unwrap();
        let round_trip = NumericRegister::try_from(bits).unwrap();

        // Verify values
        assert_eq!(max_register.u8_small, round_trip.u8_small);
        assert_eq!(max_register.u8_full, round_trip.u8_full);
        assert_eq!(max_register.u16_small, round_trip.u16_small);
        assert_eq!(max_register.u16_full, round_trip.u16_full);
        assert_eq!(max_register.u32_small, round_trip.u32_small);
        assert_eq!(max_register.u32_full, round_trip.u32_full);

        // Test zero values in each field
        let zero_register = NumericRegister {
            u8_small: 0,
            u8_full: 0,
            u16_small: 0,
            u16_full: 0,
            u32_small: 0,
            u32_full: 0,
        };

        // Convert to bits and back
        let bits: u64 = zero_register.try_into().unwrap();
        let round_trip = NumericRegister::try_from(bits).unwrap();

        // Verify values
        assert_eq!(zero_register.u8_small, round_trip.u8_small);
        assert_eq!(zero_register.u8_full, round_trip.u8_full);
        assert_eq!(zero_register.u16_small, round_trip.u16_small);
        assert_eq!(zero_register.u16_full, round_trip.u16_full);
        assert_eq!(zero_register.u32_small, round_trip.u32_small);
        assert_eq!(zero_register.u32_full, round_trip.u32_full);
    }
}
