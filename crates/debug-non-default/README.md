# DebugNonDefault

A derive macro similar to Debug but only prints fields that aren't equal to their default values.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
debug-non-default = "0.1.0"
```

Then use it in your code:

```rust
use debug_non_default::DebugNonDefault;

// Regular struct
#[derive(DebugNonDefault, Default)]
struct Person {
    name: String,
    age: u32,
    city: String,
}

// Tuple struct
#[derive(DebugNonDefault, Default)]
struct Point(i32, i32, i32);

fn main() {
    // Regular struct example
    let person = Person {
        name: "John".to_string(),
        age: 0, // Default for u32
        city: "New York".to_string(),
    };
    println!("{:?}", person); // Person { name: "John", city: "New York" }

    // Tuple struct example
    let point = Point(5, 0, 0); // Only first coordinate is non-default
    println!("{:?}", point); // Point(5, _, _)
}
```

## Features

- For regular structs, only non-default fields are displayed
- For tuple structs, all fields are shown with underscores (`_`) for default values
- Unit structs simply print their name
- When all fields are default, only the struct name is printed

## Requirements

- All fields in the struct must implement both `Debug` and `Default` traits
- Supports regular structs, tuple structs, and unit structs

## How It Works

The macro creates a custom `Debug` implementation that compares each field with its default value and only includes non-default fields in the output. For tuple structs, it maintains the structure by using underscores for default values.

## Examples

See the `examples` directory for detailed examples:
- `basic.rs`: Simple usage with primitive types
- `nested.rs`: Complex example with nested structures and collections
- `tuple_structs.rs`: Examples with tuple structs and unit structs

Run an example with:
```bash
cargo run --example basic
```

## Testing

Tests are located in the `tests` directory as integration tests. Run them with:
```bash
cargo test
```

## License

MIT
