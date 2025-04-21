//! # Debug Non Default
//!
//! A procedural macro crate that provides a custom `Debug` implementation which only
//! displays fields that differ from their default values.
//!
//! ## Overview
//!
//! When debugging complex data structures, it's often helpful to focus only on the
//! non-default or meaningful values rather than seeing every field. This crate provides
//! the `DebugNonDefault` derive macro that automatically implements a specialized
//! version of the standard `Debug` trait that only displays fields that are different
//! from their default values.
//!
//! ## Features
//!
//! - For regular structs, only non-default fields are displayed
//! - For tuple structs, all fields are shown but default values appear as underscores (`_`)
//! - Unit structs simply print their name
//! - When all fields are default, only the struct name is printed
//!
//! ## Requirements
//!
//! - All fields in the struct must implement both `Debug` and `Default` traits
//! - Works with regular structs, tuple structs, and unit structs
//! - Enums are not supported
//!
//! ## Usage
//!
//! ```rust
//! use debug_non_default::DebugNonDefault;
//!
//! #[derive(DebugNonDefault, Default)]
//! struct Configuration {
//!     hostname: String,
//!     port: u16,          // Will default to 0
//!     timeout_seconds: u32, // Will default to 0
//!     retry_count: u8,    // Will default to 0
//! }
//!
//! // Only configured fields will appear in debug output
//! let config = Configuration {
//!     hostname: "example.com".to_string(),
//!     port: 8080,
//!     ..Default::default()
//! };
//!
//! // Prints: Configuration { hostname: "example.com", port: 8080 }
//! println!("{:?}", config);
//!
//! // Using with tuple structs
//! #[derive(DebugNonDefault, Default)]
//! struct Point(i32, i32, i32);
//!
//! // Only the y-coordinate is non-default
//! let point = Point(0, 10, 0);
//!
//! // Prints: Point(_, 10, _)
//! println!("{:?}", point);
//! ```
//!
//! ## Implementation Details
//!
//! The macro generates a custom `Debug` implementation that compares each field with
//! its default value using `!=` and only includes non-default fields in the output.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Index};

/// A derive macro similar to Debug but only prints fields that are not equal to their default values.
///
/// This requires that all fields implement both Debug and Default traits.
#[proc_macro_derive(DebugNonDefault)]
pub fn derive_debug_non_default(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Get the data struct
    if let Data::Struct(data_struct) = input.data {
        match data_struct.fields {
            Fields::Named(named_fields) => {
                // Handle named fields (regular structs)
                let field_debugs = named_fields.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_name_str = field_name.to_string();
                    let field_type = field.ty.clone();

                    quote! {
                        if self.#field_name != <#field_type>::default() {
                            debug_struct.field(#field_name_str, &self.#field_name);
                        }
                    }
                });

                let expanded = quote! {
                    impl ::core::fmt::Debug for #name {
                        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                            let mut debug_struct = f.debug_struct(stringify!(#name));
                            #(#field_debugs)*
                            debug_struct.finish()
                        }
                    }
                };

                TokenStream::from(expanded)
            }
            Fields::Unnamed(unnamed_fields) => {
                // Handle tuple structs
                let field_debugs = unnamed_fields.unnamed.iter().enumerate().map(|(i, field)| {
                    let index = Index::from(i);
                    let field_type = field.ty.clone();

                    quote! {
                        if self.#index != <#field_type>::default() {
                            debug_tuple.field(&self.#index);
                        } else {
                            debug_tuple.field(&format_args!("_"));
                        }
                    }
                });

                let expanded = quote! {
                    impl ::core::fmt::Debug for #name {
                        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                            let mut debug_tuple = f.debug_tuple(stringify!(#name));
                            #(#field_debugs)*
                            debug_tuple.finish()
                        }
                    }
                };

                TokenStream::from(expanded)
            }
            Fields::Unit => {
                // For unit structs, just implement a basic Debug
                let expanded = quote! {
                    impl ::core::fmt::Debug for #name {
                        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                            f.write_str(stringify!(#name))
                        }
                    }
                };

                TokenStream::from(expanded)
            }
        }
    } else {
        // We don't support enums or unions
        TokenStream::from(quote! {
            compile_error!("DebugNonDefault only supports structs");
        })
    }
}
