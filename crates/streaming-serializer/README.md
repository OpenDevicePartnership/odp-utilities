# streaming-serializer

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An incremental buffer writer for `no_std` environments with async support. This crate provides a way to serialize data into fixed-size buffers, yielding chunks of data as they're produced - perfect for streaming protocols and memory-constrained environments.

## Features

- **`no_std` compatible** - Works entirely with stack-allocated buffers
- **Zero allocation** - No heap allocations required
- **Async/await support** - Uses Rust's async syntax for natural backpressure handling
- **Incremental streaming** - Yields data chunks as the buffer fills up
- **Type-safe serialization** - Generic over data types with compile-time guarantees
- **Safe Rust only** - No unsafe code used anywhere

## Use Cases

This crate is particularly useful for:

- **Embedded systems** where memory is limited and heap allocation is avoided
- **Network protocols** that need to stream data incrementally
- **File formats** where you want to write data progressively without buffering everything in memory
- **Real-time systems** where predictable memory usage is important

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
streaming-serializer = "0.1.0"
```

## Basic Usage

```rust
use streaming_serializer::Serializer;
use core::pin::pin;

// Create a small buffer for demonstration
let mut buffer = [0u8; 4];
let serializer = Serializer::new(&mut buffer);

// Create a writer that will serialize some data
let mut chunks = pin!(writer.serialize(async move |mut buf| {
    buf.write_u8(0x01).await;
    buf.write_u8(0x02).await;
    buf.write_u8(0x03).await;
    buf.write_u8(0x04).await;
    buf.write_u8(0x05).await; // This will require a second chunk
}));

// Get the first chunk of data
if let Some(chunk) = chunks.next() {
    println!("First chunk: {:?}", &*chunk); // [1, 2, 3, 4]
}

// Get the second chunk
if let Some(chunk) = chunks.next() {
    println!("Second chunk: {:?}", &*chunk); // [5]
}

// No more data available
assert!(chunks.next().is_none());
```

## Advanced Examples

### Working with Different Data Types

```rust
use streaming_serializer::Serializer;
use core::pin::pin;

let mut buffer = [0u8; 16];
let serializer = Serializer::new(&mut buffer);

let mut chunks = pin!(writer.serialize(async move |mut buf| {
    // Write different numeric types with specific endianness
    buf.write_num_be::<u16>(0x1234).await;      // Big-endian u16: [0x12, 0x34]
    buf.write_num_le::<u32>(0x56789ABC).await;  // Little-endian u32: [0xBC, 0x9A, 0x78, 0x56]

    // Write byte slices directly
    buf.write_slice(b"hello").await;

    // Mix different operations
    buf.write_u8(0xFF).await;
    buf.write_num_be::<i16>(-1).await;          // Signed types also supported
}));

while let Some(chunk) = chunks.next() {
    println!("Chunk: {:?}", &*chunk);
}
```

### Serializing Custom Types

```rust
use streaming_serializer::Serializer;
use core::pin::pin;

#[derive(Debug)]
struct Packet {
    version: u8,
    message_type: u16,
    payload: &'static [u8],
}

impl Packet {
    async fn serialize(&self, mut writer: streaming_serializer::Buffer<'_>) {
        // Write header
        writer.write_u8(self.version).await;
        writer.write_num_be(self.message_type).await;
        writer.write_num_be(self.payload.len() as u16).await;

        // Write payload
        writer.write_slice(self.payload).await;
    }
}

let packet = Packet {
    version: 1,
    message_type: 0x0100,
    payload: b"Hello, world!",
};

let mut buffer = [0u8; 64];
let serializer = Serializer::new(&mut buffer);
let mut chunks = pin!(serializer.serialize(async move |buf| {
    packet.serialize(buf).await;
}));

// Process serialized packet in chunks
while let Some(chunk) = chunks.next() {
    send_over_network(&chunk);
}

fn send_over_network(_data: &[u8]) {
    // Your network code here
}
```

## API Overview

### Core Functions

Ò- **`Serializer::new(buffer)`** - Creates a new `WriterBuilder` using the provided buffer
- **`serializer.serialize(closure)`** - Creates a `Writer` with serialization logic
- **`chunks.next()`** - Returns the next chunk of data or `None` when complete

### Buffer Methods

The `Buffer` type provides async methods for writing data:

- **`write_u8(value)`** - Write a single byte
- **`write_slice(data)`** - Write a slice of bytes
- **`write_num_be<T>(value)`** - Write numeric type in big-endian format
- **`write_num_le<T>(value)`** - Write numeric type in little-endian format

Supported numeric types: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`

## How It Works

The incremental writer uses Rust's async/await system to implement cooperative yielding:

1. You provide a buffer and serialization logic as an async closure
2. The serialization runs until the buffer is full, then yields (`Poll::Pending`)
3. Calling `chunks.next()` returns the current buffer contents and resets the position
4. The serialization resumes from where it left off
5. This continues until the serialization completes

The key insight is that async functions naturally pause at await points when resources (buffer space) are unavailable, making incremental processing seamless.

## Error Handling

This crate uses Rust's type system to prevent errors at compile time:

- **Buffer overruns are impossible** - The async system handles backpressure automatically
- **Memory safety** - All operations use safe Rust with proper lifetime management
- **No panics** - The only panic case is misusing the `RefCell` borrowing (documented in API)
- **No unsafe code** - Safety doesn't compromise performance

## Performance

- **Zero allocation** - Everything uses stack-allocated buffers
- **Minimal overhead** - Async state machines compile to efficient code
- **Predictable** - Memory usage is determined by your buffer size
- **No unsafe code** - Safety doesn't compromise performance

## Limitations

- **Buffer size determines chunk granularity** - Smaller buffers mean more chunks
- **Sequential processing** - Chunks must be consumed in order
- **Single-threaded** - Uses `RefCell` for interior mutability (typical for `no_std`)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
