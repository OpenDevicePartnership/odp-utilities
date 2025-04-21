#![allow(missing_docs)]

use debug_non_default::DebugNonDefault;

#[derive(DebugNonDefault, Default)]
struct Person {
    name: String,
    age: u32,
    city: String,
}

#[test]
fn test_empty_struct() {
    // All default values, should print empty struct
    let person = Person::default();
    assert_eq!(format!("{:?}", person), "Person");
}

#[test]
fn test_partial_fields() {
    // Some non-default values
    let person = Person {
        name: "John".to_string(),
        age: 0, // Default for u32
        city: "New York".to_string(),
    };
    assert_eq!(
        format!("{:?}", person),
        "Person { name: \"John\", city: \"New York\" }"
    );
}

#[test]
fn test_all_fields() {
    // All non-default values
    let person = Person {
        name: "Jane".to_string(),
        age: 30,
        city: "Boston".to_string(),
    };
    assert_eq!(
        format!("{:?}", person),
        "Person { name: \"Jane\", age: 30, city: \"Boston\" }"
    );
}
