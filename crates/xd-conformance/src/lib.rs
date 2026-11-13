//! The conformance suite (docs/testing.md, docs/extensibility.md "Reference
//! tests"): proptest strategies that generate xD values, and the laws those
//! values must satisfy, written once so that every built-in scheme, every
//! Nickel configuration and, later, every plugin runs the same checks.
//!
//! A law is a function returning `Result<(), TestCaseError>`, so it can be
//! called from inside any `proptest!` block:
//!
//! ```ignore
//! // In a test file of the crate that defines the scheme or projection:
//! use proptest::prelude::*;
//! use xd_conformance::{laws, strategies};
//!
//! proptest! {
//!     #[test]
//!     fn my_scheme_round_trips((s, a) in strategies::scheme_and_address()) {
//!         laws::put_get(&s, &a)?;
//!     }
//! }
//! ```

pub mod laws;
pub mod strategies;

/// Property cases: `PROPTEST_CASES` when it is set (the weekly CI run sets
/// 10,000), otherwise `default`.
pub fn cases(default: u32) -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
