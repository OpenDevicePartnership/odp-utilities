#![allow(missing_docs)]

use debug_non_default::DebugNonDefault;

// Tuple struct with all field types supporting Default
#[derive(DebugNonDefault, Default)]
struct Point(i32, i32, i32);

// Tuple struct with String fields
#[derive(DebugNonDefault, Default)]
struct RGB(u8, u8, u8);

// Unit struct
#[derive(DebugNonDefault, Default)]
struct Empty;

#[test]
fn test_all_default_tuple() {
    let point = Point::default(); // (0, 0, 0)
    assert_eq!(format!("{:?}", point), "Point(_, _, _)");
}

#[test]
fn test_partial_non_default_tuple() {
    let point = Point(1, 0, 0); // First field is non-default
    assert_eq!(format!("{:?}", point), "Point(1, _, _)");
}

#[test]
fn test_some_non_default_tuple() {
    let point = Point(1, 0, 2); // First and third fields are non-default
    assert_eq!(format!("{:?}", point), "Point(1, _, 2)");
}

#[test]
fn test_all_non_default_tuple() {
    let point = Point(1, 2, 3); // All fields are non-default
    assert_eq!(format!("{:?}", point), "Point(1, 2, 3)");
}

#[test]
fn test_rgb_tuple() {
    let black = RGB::default(); // (0, 0, 0)
    assert_eq!(format!("{:?}", black), "RGB(_, _, _)");

    let red = RGB(255, 0, 0);
    assert_eq!(format!("{:?}", red), "RGB(255, _, _)");

    let purple = RGB(255, 0, 255);
    assert_eq!(format!("{:?}", purple), "RGB(255, _, 255)");
}

#[test]
fn test_unit_struct() {
    let empty = Empty;
    assert_eq!(format!("{:?}", empty), "Empty");
}
