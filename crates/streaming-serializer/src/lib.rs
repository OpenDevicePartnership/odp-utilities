//! An incremental writer for `no_std` environments with async support.
//!
//! This crate provides a way to serialize data incrementally into a fixed-size buffer,
//! yielding chunks of data as they're produced. This is particularly useful for streaming
//! protocols or when working with memory-constrained environments where you need to
//! process data in small chunks.
//!
//! # Features
//!
//! - **No allocation**: Works entirely with stack-allocated buffers
//! - **Async support**: Uses Rust's async/await syntax for backpressure handling
//! - **Incremental processing**: Yields data chunks as they're produced
//! - **Type-safe**: Generic over data types with compile-time guarantees
//! - **No unsafe**: Uses safe Rust for all operations
//!
//! # Basic Usage
//!
//! ```rust
//! use streaming_serializer::Serializer;
//! use core::pin::pin;
//!
//! // Create a small buffer for demonstration
//! let mut buffer = [0u8; 4];
//! let serializer = Serializer::new(&mut buffer);
//!
//! // Create a serializer that will serialize some data
//! let mut chunks = pin!(serializer.serialize(async move |mut buf| {
//!     buf.write_u8(0x01).await;
//!     buf.write_u8(0x02).await;
//!     buf.write_u8(0x03).await;
//!     buf.write_u8(0x04).await;
//!     buf.write_u8(0x05).await; // This will require a second chunk
//! }));
//!
//! // Get the first chunk of data
//! if let Some(chunk) = chunks.next() {
//!     println!("First chunk: {:?}", &*chunk); // [1, 2, 3, 4]
//! }
//!
//! // Get the second chunk
//! if let Some(chunk) = chunks.next() {
//!     println!("Second chunk: {:?}", &*chunk); // [5]
//! }
//!
//! // No more data
//! assert!(chunks.next().is_none());
//! ```

#![no_std]

mod buffer;
mod shared_state;
mod write_byte_future;
mod write_slice_future;

pub use buffer::*;
use core::cell::{Ref, RefCell};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use pin_project::pin_project;
use shared_state::SharedState;

/// A pinned state that can be used to retrieve chunks of data from the serializer.
#[pin_project]
pub struct PinnedState<'buf, F>
where
    F: Future<Output = ()> + 'buf,
{
    #[pin]
    fut: F,
    done: bool,
    shared: &'buf RefCell<SharedState<'buf>>,
}

impl<'buf, F> PinnedState<'buf, F>
where
    F: Future<Output = ()> + 'buf,
{
    /// Retrieves the next chunk of data from the serializer.
    ///
    /// This method will return `None` if the serializer has completed.
    pub fn next(self: &mut Pin<&mut Self>) -> Option<Ref<'buf, [u8]>> {
        if self.done {
            return None;
        }

        let waker = Waker::noop().clone();
        let mut cx = Context::from_waker(&waker);
        let mut this = self.as_mut().project();
        this.shared.borrow_mut().offset = 0;

        match this.fut.as_mut().poll(&mut cx) {
            Poll::Ready(()) => {
                *this.done = true;
            }
            Poll::Pending => {}
        }

        let shared = this.shared.borrow();
        let n = shared.offset;
        if n == 0 {
            return None;
        }
        let buffer = Ref::map(shared, |b| &b.buffer[..n]);
        Some(buffer)
    }
}

/// The `Serializer` is created by [`Serializer::new`] and provides the [`Serializer::serialize`]
/// method to create a [`PinnedState`] instance with your serialization logic.
///
/// # Examples
///
/// ```rust
/// use streaming_serializer::Serializer;
/// use core::pin::pin;
///
/// let mut buffer = [0u8; 10];
/// let serializer = Serializer::new(&mut buffer);
///
/// let chunks = pin!(serializer.serialize(async move |mut buf| {
///     buf.write_u8(42).await;
///     buf.write_slice(b"hello").await;
/// }));
/// ```
#[pin_project]
pub struct Serializer<'buf> {
    shared: RefCell<SharedState<'buf>>,
}
impl<'buf> Serializer<'buf> {
    /// Creates a new [`Serializer`] that uses the provided buffer for serialization.
    ///
    /// This is the main entry point for the incremental buffer writer. It takes a mutable
    /// slice that will be used as the internal buffer for chunk generation and returns
    /// a builder that can create writers with specific serialization logic.
    ///
    /// The buffer size determines how much data can be accumulated before yielding a chunk.
    /// Smaller buffers will result in more frequent chunk yields but lower memory usage,
    /// while larger buffers will result in fewer, larger chunks.
    pub fn new(buf: &'buf mut [u8]) -> Self {
        Self {
            shared: RefCell::new(SharedState {
                buffer: buf,
                offset: 0,
            }),
        }
    }

    /// Creates a [`Serializer`] with the provided serialization logic.
    ///
    /// This method takes a closure that receives a [`Buffer`] and returns a future
    /// that performs the serialization. The closure is called immediately to create
    /// the serialization future, which is then managed by the returned [`Serializer`].
    pub fn serialize<B, F>(&'buf self, build: B) -> PinnedState<'buf, F>
    where
        B: FnOnce(Buffer<'buf>) -> F,
        F: Future<Output = ()> + 'buf,
    {
        PinnedState {
            shared: &self.shared,
            done: false,
            fut: build(Buffer::new(&self.shared)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::pin::pin;

    #[derive(Debug, Clone, Copy)]
    struct MyData {
        field1: u8,
        field2: u16,
        field3: u32,
        field4: u64,
    }

    async fn serialize_my_data(data: MyData, mut writer: Buffer<'_>) {
        writer.write_u8(data.field1).await;
        for byte in data.field2.to_be_bytes() {
            writer.write_u8(byte).await;
        }
        for byte in data.field3.to_be_bytes() {
            writer.write_u8(byte).await;
        }
        for byte in data.field4.to_be_bytes() {
            writer.write_u8(byte).await;
        }
    }

    #[test]
    fn test_with_callback() {
        let mut buf = [0u8; 4];
        let my_data = MyData {
            field1: 0x01,
            field2: 0x0203,
            field3: 0x04050607,
            field4: 0x08090A0B0C0D0E0F,
        };

        let serializer = Serializer::new(&mut buf);
        let mut chunks =
            pin!(serializer.serialize(async move |b| serialize_my_data(my_data, b).await));

        assert_eq!(
            chunks.next().as_deref(),
            Some([0x01, 0x02, 0x03, 0x04].as_slice())
        );
        assert_eq!(
            chunks.next().as_deref(),
            Some([0x05, 0x06, 0x07, 0x08].as_slice())
        );
        assert_eq!(
            chunks.next().as_deref(),
            Some([0x09, 0x0A, 0x0B, 0x0C].as_slice())
        );
        assert_eq!(
            chunks.next().as_deref(),
            Some([0x0D, 0x0E, 0x0F].as_slice())
        );
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    fn test_multiple_writers() {
        let mut buf1 = [0u8; 4];
        let mut buf2 = [0u8; 4];

        let data1 = MyData {
            field1: 0x01,
            field2: 0x0203,
            field3: 0x04050607,
            field4: 0x08090A0B0C0D0E0F,
        };

        let data2 = MyData {
            field1: 0x10,
            field2: 0x2030,
            field3: 0x40506070,
            field4: 0x8090A0B0C0D0E0F0,
        };

        let serializer1 = Serializer::new(&mut buf1);
        let serializer2 = Serializer::new(&mut buf2);
        let mut chunks1 =
            pin!(serializer1.serialize(async move |b| serialize_my_data(data1, b).await));
        let mut chunks2 =
            pin!(serializer2.serialize(async move |b| serialize_my_data(data2, b).await));

        assert_eq!(
            chunks1.next().as_deref(),
            Some([0x01, 0x02, 0x03, 0x04].as_slice())
        );
        assert_eq!(
            chunks2.next().as_deref(),
            Some([0x10, 0x20, 0x30, 0x40].as_slice())
        );
        assert_eq!(
            chunks1.next().as_deref(),
            Some([0x05, 0x06, 0x07, 0x08].as_slice())
        );
        assert_eq!(
            chunks2.next().as_deref(),
            Some([0x50, 0x60, 0x70, 0x80].as_slice())
        );
    }

    #[test]
    fn test_zero_length_buffer() {
        let mut buf: [u8; 0] = [];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |_b| {
            // attempt writes that will always pend due to zero capacity
        }));
        assert_eq!(chunks.next().as_deref(), None);
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    fn test_single_byte_buffer_streaming() {
        let mut buf = [0u8; 1];
        let data = MyData {
            field1: 0xAA,
            field2: 0,
            field3: 0,
            field4: 0,
        };
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |mut b| {
            b.write_u8(data.field1).await;
        }));
        assert_eq!(chunks.next().as_deref(), Some([0xAA].as_slice()));
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    fn test_exact_fit_chunk() {
        let mut buf = [0u8; 4];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |mut b| {
            for v in [1u8, 2, 3, 4] {
                b.write_u8(v).await;
            }
        }));
        assert_eq!(chunks.next().as_deref(), Some([1, 2, 3, 4].as_slice()));
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    fn test_empty_serializer() {
        let mut buf = [0u8; 4];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |_b| {
            // do nothing
        }));
        assert_eq!(chunks.next().as_deref(), None);
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    fn test_idempotent_after_done() {
        let mut buf = [0u8; 2];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |mut b| {
            b.write_u8(7).await;
            b.write_u8(8).await;
        }));
        assert_eq!(chunks.next().as_deref(), Some([7, 8].as_slice()));
        assert_eq!(chunks.next().as_deref(), None);
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    #[should_panic]
    fn test_next_while_holding_previous_slice_panics() {
        let mut buf = [0u8; 1];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |mut b| {
            b.write_u8(1).await;
            b.write_u8(2).await; // will require a second chunk
        }));
        let first = chunks.next().unwrap();
        // This should panic due to RefCell borrow rules
        let _second = chunks.next();
        drop(first);
    }

    #[test]
    fn test_write_slice_exact_fit() {
        let mut buf = [0u8; 4];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |mut b| {
            let src = [1u8, 2, 3, 4];
            b.write_slice(&src).await;
        }));
        assert_eq!(chunks.next().as_deref(), Some([1, 2, 3, 4].as_slice()));
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    fn test_write_slice_multi_chunk() {
        let mut buf = [0u8; 4];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |mut b| {
            let src = [1u8, 2, 3, 4, 5, 6, 7];
            b.write_slice(&src).await;
        }));
        assert_eq!(chunks.next().as_deref(), Some([1, 2, 3, 4].as_slice()));
        assert_eq!(chunks.next().as_deref(), Some([5, 6, 7].as_slice()));
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    fn test_write_num_be_le_u16() {
        let mut buf = [0u8; 2];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |mut b| {
            b.write_num_be::<u16>(0x1234).await;
            b.write_num_le::<u16>(0x5678).await;
        }));
        assert_eq!(
            chunks.next().as_deref(),
            Some(0x1234u16.to_be_bytes().as_slice())
        );
        assert_eq!(
            chunks.next().as_deref(),
            Some(0x5678u16.to_le_bytes().as_slice())
        );
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    fn test_write_num_be_le_i16() {
        let mut buf = [0u8; 2];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |mut b| {
            b.write_num_be::<i16>(-2).await;
            b.write_num_le::<i16>(-3).await;
        }));
        assert_eq!(
            chunks.next().as_deref(),
            Some((-2i16).to_be_bytes().as_slice())
        );
        assert_eq!(
            chunks.next().as_deref(),
            Some((-3i16).to_le_bytes().as_slice())
        );
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    fn test_write_num_be_u32_exact() {
        let mut buf = [0u8; 4];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |mut b| {
            b.write_num_be::<u32>(0x01020304).await;
        }));
        assert_eq!(
            chunks.next().as_deref(),
            Some(0x01020304u32.to_be_bytes().as_slice())
        );
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    fn test_write_num_le_u64_two_chunks() {
        let mut buf = [0u8; 4];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |mut b| {
            b.write_num_le::<u64>(0x0102030405060708).await;
        }));
        let first = &0x0102030405060708u64.to_le_bytes()[..4];
        let second = &0x0102030405060708u64.to_le_bytes()[4..];
        assert_eq!(chunks.next().as_deref(), Some(first));
        assert_eq!(chunks.next().as_deref(), Some(second));
        assert_eq!(chunks.next().as_deref(), None);
    }

    #[test]
    fn test_write_num_mixed_types() {
        let mut buf = [0u8; 3];
        let serializer = Serializer::new(&mut buf);
        let mut chunks = pin!(serializer.serialize(async move |mut b| {
            b.write_num_be::<u16>(0x1122).await; // fills first 2 bytes
            b.write_num_le::<i32>(0x33445566).await; // produces more; next() should include only 1 byte here, then rest
        }));
        // First chunk: [0x11,0x22, 0x66]
        let exp1 = [0x11u8, 0x22u8, 0x33445566i32.to_le_bytes()[0]];
        assert_eq!(chunks.next().as_deref(), Some(exp1.as_slice()));
        // Remaining LE bytes
        let rem = &0x33445566i32.to_le_bytes()[1..];
        assert_eq!(chunks.next().as_deref(), Some(rem[..3].as_ref()));
        let leftover = &rem[3..];
        if !leftover.is_empty() {
            assert_eq!(chunks.next().as_deref(), Some(leftover));
        }
        assert_eq!(chunks.next().as_deref(), None);
    }
}
