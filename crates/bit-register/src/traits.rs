use num_traits::{One, Zero};

/// Trait for types that are a bit register which can be converted to and from an unsigned integer type.
pub trait BitRegister<T>:
    Sized + TryFrom<T, Error = &'static str> + TryInto<T, Error = &'static str>
{
}

/// Trait for reflecting the number of bytes for the underlying type
pub trait NumBytes {
    /// Number of bytes for the underlying type
    const NUM_BYTES: usize;
}

/// Trait for types that can be converted to a bit pattern (an unsigned integer)
pub trait TryIntoBits<T>: Sized {
    /// Try to convert the type to a bit pattern (unsigned integer)
    fn try_into_bits(self) -> Result<T, &'static str>;
}

/// Trait for types that can be converted from a bit pattern (an unsigned integer)
pub trait TryFromBits<T>: Sized {
    /// Try to convert a bit pattern (unsigned integer) to the target type
    fn try_from_bits(bits: T) -> Result<Self, &'static str>;
}

macro_rules! impl_try_into_from_bits {
    ($($t:ty => $num_bytes:literal),*) => {
        $(
            impl NumBytes for $t {
                const NUM_BYTES: usize = $num_bytes;
            }
            impl<T: TryFrom<$t>> TryIntoBits<T> for $t {
                fn try_into_bits(self) -> Result<T, &'static str> {
                    TryInto::try_into(self).map_err(|_| concat!(stringify!($t), " value too large for target type"))
                }
            }
            impl<T> TryFromBits<T> for $t where $t: TryFrom<T> {
                fn try_from_bits(bits: T) -> Result<Self, &'static str> {
                    TryFrom::try_from(bits).map_err(|_| concat!("bit pattern too large for target type ", stringify!($t)))
                }
            }
        )+
    }
}

impl_try_into_from_bits!(u8 => 1, u16 => 2, u32 => 4, u64 => 8);

// Bool gets its own special impls
impl NumBytes for bool {
    const NUM_BYTES: usize = 1;
}
impl<T: One + Zero> TryIntoBits<T> for bool {
    fn try_into_bits(self) -> Result<T, &'static str> {
        Ok(if self { One::one() } else { Zero::zero() })
    }
}
impl<T: One + Zero + PartialEq<T>> TryFromBits<T> for bool {
    fn try_from_bits(bits: T) -> Result<Self, &'static str> {
        if bits == One::one() {
            Ok(true)
        } else if bits == Zero::zero() {
            Ok(false)
        } else {
            Err("bit pattern too large for target type bool")
        }
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use std::format;

    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_try_into_bits() {
        let value: u8 = 0b1010;
        let bits: u8 = value.try_into_bits().unwrap();
        assert_eq!(bits, 0b1010);
    }

    #[test]
    fn test_bool() {
        assert_eq!(TryIntoBits::<u8>::try_into_bits(false).unwrap(), 0u8);
        assert_eq!(TryIntoBits::<u8>::try_into_bits(true).unwrap(), 1u8);
        assert_eq!(
            <bool as TryFromBits<u8>>::try_from_bits(0u8).unwrap(),
            false
        );
        assert_eq!(<bool as TryFromBits<u8>>::try_from_bits(1u8).unwrap(), true);
        assert_eq!(<bool as TryFromBits<u8>>::try_from_bits(2u8).is_err(), true);
    }

    proptest! {
        #[test]
        fn prop_try_into_bits_identity_u8(val: u8) {
            let bits: u8 = val.try_into_bits().unwrap();
            prop_assert_eq!(bits, val);
        }

        #[test]
        fn prop_try_into_bits_identity_u16(val: u16) {
            let bits: u16 = val.try_into_bits().unwrap();
            prop_assert_eq!(bits, val);
        }

        #[test]
        fn prop_try_into_bits_identity_u32(val: u32) {
            let bits: u32 = val.try_into_bits().unwrap();
            prop_assert_eq!(bits, val);
        }

        #[test]
        fn prop_try_into_bits_u8_to_u16(val: u8) {
            let bits: u16 = val.try_into_bits().unwrap();
            prop_assert_eq!(bits, val as u16);
        }

        #[test]
        fn prop_try_into_bits_u16_to_u32(val: u16) {
            let bits: u32 = val.try_into_bits().unwrap();
            prop_assert_eq!(bits, val as u32);
        }

        #[test]
        fn prop_try_into_bits_u16_to_u8(val: u16) {
            let result: Result<u8, _> = val.try_into_bits();
            if val <= u8::MAX as u16 {
                prop_assert_eq!(result.unwrap(), val as u8);
            } else {
                prop_assert_eq!(result.unwrap_err(), "u16 value too large for target type");
            }
        }

        #[test]
        fn prop_try_into_bits_u32_to_u16(val: u32) {
            let result: Result<u16, _> = val.try_into_bits();
            if val <= u16::MAX as u32 {
                prop_assert_eq!(result.unwrap(), val as u16);
            } else {
                prop_assert_eq!(result.unwrap_err(), "u32 value too large for target type");
            }
        }

        #[test]
        fn prop_try_from_bits_identity_u8(bits: u8) {
            let val: u8 = TryFromBits::try_from_bits(bits).unwrap();
            prop_assert_eq!(val, bits);
        }

        #[test]
        fn prop_try_from_bits_identity_u16(bits: u16) {
            let val: u16 = TryFromBits::try_from_bits(bits).unwrap();
            prop_assert_eq!(val, bits);
        }

        #[test]
        fn prop_try_from_bits_identity_u32(bits: u32) {
            let val: u32 = TryFromBits::try_from_bits(bits).unwrap();
            prop_assert_eq!(val, bits);
        }

        #[test]
        fn prop_try_from_bits_u8_to_u16(bits: u8) {
            let val: u16 = TryFromBits::try_from_bits(bits).unwrap();
            prop_assert_eq!(val, bits as u16);
        }

        #[test]
        fn prop_try_from_bits_u16_to_u32(bits: u16) {
            let val: u32 = TryFromBits::try_from_bits(bits).unwrap();
            prop_assert_eq!(val, bits as u32);
        }

        #[test]
        fn prop_try_from_bits_u16_to_u8(bits: u16) {
            let result: Result<u8, _> = TryFromBits::try_from_bits(bits);
             if bits <= u8::MAX as u16 {
                prop_assert_eq!(result.unwrap(), bits as u8);
            } else {
                prop_assert_eq!(result.unwrap_err(), "bit pattern too large for target type u8");
            }
        }

         #[test]
        fn prop_try_from_bits_u32_to_u16(bits: u32) {
            let result: Result<u16, _> = TryFromBits::try_from_bits(bits);
             if bits <= u16::MAX as u32 {
                prop_assert_eq!(result.unwrap(), bits as u16);
            } else {
                prop_assert_eq!(result.unwrap_err(), "bit pattern too large for target type u16");
            }
        }
    }
}
