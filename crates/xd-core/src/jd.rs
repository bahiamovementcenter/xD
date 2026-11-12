//! Johnny.Decimal as a configuration of the core: the defaults.
//!
//! These will move to a Nickel configuration (ADR 0001, step 3); they live
//! here until `nickel-embed` loads them.

use crate::address::Address;
use crate::scheme::{Level, Scheme};

/// Base 10: an area digit, a category digit, then a two-digit ID after a
/// dot. Areas are written as the span of their categories, `10-19`.
pub fn scheme() -> Scheme {
    Scheme {
        base: 10,
        levels: vec![
            Level {
                name: "area".into(),
                width: 1,
                separator: String::new(),
                range: true,
            },
            Level {
                name: "category".into(),
                width: 1,
                separator: String::new(),
                range: false,
            },
            Level {
                name: "id".into(),
                width: 2,
                separator: ".".into(),
                range: false,
            },
        ],
    }
}

/// The first regular ID in a category. JD's docs start at `.11` and reserve
/// `.00`–`.09`; `.10` is treated as reserved too (formal open question 1).
pub const FIRST_ID: u32 = 11;

/// The standard zeros (formal 2.1), as numbers allocation skips: area `00-09`
/// (the system's), category `x0` (each area's) and IDs `.00`–`.10`. They
/// exist, but are made on purpose, never handed out as "the next one".
pub fn standard_zeros(address: &Address) -> bool {
    match address.values() {
        [0] => true,
        [_, 0] => true,
        [_, _, id] => *id < FIRST_ID,
        _ => false,
    }
}
