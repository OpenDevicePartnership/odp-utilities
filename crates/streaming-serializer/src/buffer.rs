use core::cell::RefCell;

use crate::{
    SharedState, write_byte_future::WriteByteFuture, write_slice_future::WriteSliceFuture,
};

/// A buffer that provides async methods for writing data incrementally.
///
/// The `Buffer` allows you to write various types of data (bytes, slices, numeric types)
/// using async methods. When the internal buffer becomes full, the write operations will
/// yield a chunk of data to the caller.
///
/// # Examples
///
/// ```rust
/// use streaming_serializer::Serializer;
/// use core::pin::pin;
///
/// let mut buffer = [0u8; 8];
/// let serializer = Serializer::new(&mut buffer);
///
/// let mut chunks = pin!(serializer.serialize(async move |mut buf| {
///     buf.write_u8(42).await;
///     buf.write_num_be::<u16>(0x1234).await;
///     buf.write_slice(b"hi").await;
/// }));
///
/// while let Some(chunk) = chunks.next() { // chunks is a PinnedState
///     // Process each chunk as it becomes available
///     println!("Got chunk: {:?}", &*chunk);
/// }
/// ```
pub struct Buffer<'buf> {
    shared: &'buf RefCell<SharedState<'buf>>,
}
impl<'buf> Buffer<'buf> {
    pub(crate) fn new(shared: &'buf RefCell<SharedState<'buf>>) -> Self {
        Self { shared }
    }
}

impl<'buf> Buffer<'buf> {
    /// Writes a slice of bytes to the buffer.
    pub fn write_slice<'a>(&'a mut self, src: &'a [u8]) -> WriteSliceFuture<'buf, 'a> {
        WriteSliceFuture::new(self.shared, src)
    }

    /// Writes a single byte to the buffer.
    pub fn write_u8(&mut self, value: u8) -> WriteByteFuture<'buf> {
        WriteByteFuture::new(self.shared, value)
    }

    /// Writes a numeric value in big-endian byte order.
    ///
    /// This method converts the numeric value to its big-endian byte representation
    /// and writes it to the buffer. The value must implement the `num_traits::ToBytes`
    /// trait, which is implemented for standard integer types like `u8`, `u16`, `u32`,
    /// `u64`, `i8`, `i16`, `i32`, and `i64`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A numeric type that implements `num_traits::ToBytes`
    ///
    /// # Arguments
    ///
    /// * `value` - The numeric value to write in big-endian format
    pub async fn write_num_be<T: num_traits::ToBytes>(&mut self, value: T) -> () {
        let bytes = value.to_be_bytes();
        self.write_slice(bytes.as_ref()).await
    }

    /// Writes a numeric value in little-endian byte order.
    ///
    /// This method converts the numeric value to its little-endian byte representation
    /// and writes it to the buffer. The value must implement the `num_traits::ToBytes`
    /// trait, which is implemented for standard integer types like `u8`, `u16`, `u32`,
    /// `u64`, `i8`, `i16`, `i32`, and `i64`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A numeric type that implements `num_traits::ToBytes`
    ///
    /// # Arguments
    ///
    /// * `value` - The numeric value to write in little-endian format
    pub async fn write_num_le<T: num_traits::ToBytes>(&mut self, value: T) -> () {
        let bytes = value.to_le_bytes();
        self.write_slice(bytes.as_ref()).await
    }
}
