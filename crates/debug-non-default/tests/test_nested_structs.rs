#![allow(missing_docs)]

use debug_non_default::DebugNonDefault;
use std::collections::HashMap;

#[derive(DebugNonDefault, Default, PartialEq)]
struct Address {
    street: String,
    city: String,
    state: String,
    postal_code: String,
    country: String,
}

#[derive(DebugNonDefault, Default, PartialEq)]
struct Profile {
    bio: String,
    interests: Vec<String>,
    social_media: HashMap<String, String>,
}

#[derive(DebugNonDefault, Default)]
struct User {
    id: u64,
    username: String,
    email: String,
    address: Address,
    profile: Profile,
    active: bool,
}

#[test]
fn test_nested_default_structs() {
    // A user with default inner structs
    let user = User {
        username: "simple_user".to_string(),
        email: "simple@example.com".to_string(),
        ..Default::default()
    };
    assert_eq!(
        format!("{:?}", user),
        "User { username: \"simple_user\", email: \"simple@example.com\" }"
    );
}

#[test]
fn test_nested_non_default_structs() {
    // Create a partial address
    let address = Address {
        city: "New York".to_string(),
        country: "USA".to_string(),
        ..Default::default()
    };

    // Create a profile with some data
    let mut social_media = HashMap::new();
    social_media.insert("twitter".to_string(), "@example".to_string());

    let profile = Profile {
        bio: "Rust programmer".to_string(),
        social_media,
        ..Default::default()
    };

    // A user with nested non-default fields
    let user = User {
        username: "complex_user".to_string(),
        email: "complex@example.com".to_string(),
        address,
        profile,
        active: true,
        ..Default::default()
    };

    let debug_str = format!("{:?}", user);

    // Check that it contains all the non-default fields
    assert!(debug_str.contains("username: \"complex_user\""));
    assert!(debug_str.contains("email: \"complex@example.com\""));
    assert!(debug_str.contains("city: \"New York\""));
    assert!(debug_str.contains("country: \"USA\""));
    assert!(debug_str.contains("bio: \"Rust programmer\""));
    assert!(debug_str.contains("twitter"));
    assert!(debug_str.contains("@example"));
    assert!(debug_str.contains("active: true"));
}
