//! A numbering scheme: the base and the levels of an address (formal 1.1).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Digits above 9 are letters, so a base can go up to 36 and stay
/// case-insensitive (experiments E1: above 36 sorting and case-insensitive
/// disks break).
const ALPHABET: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const MAX_BASE: u32 = 36;

/// How numbers are built and written. Johnny.Decimal is one value of this
/// (see [`crate::jd::scheme`]); nothing in the core assumes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scheme {
    pub base: u32,
    /// From the top of the tree down. An address has one value per level,
    /// from the first level to its own.
    pub levels: Vec<Level>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    /// Digits this level adds to the number.
    pub width: u32,
    /// Written between the previous level's digits and this one's: "" for
    /// JD's category (`11`), "." for its ID (`11.11`).
    #[serde(default)]
    pub separator: String,
    /// Written as the span of its children rather than its own digits, as JD
    /// writes area `1` as `10-19`. Only for a level that has children.
    #[serde(default)]
    pub range: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SchemeError {
    #[error("base {0} is outside 2..={MAX_BASE}")]
    Base(u32),
    #[error("a scheme needs at least one level")]
    NoLevels,
    #[error("level `{0}` has width 0")]
    Width(String),
    #[error("level `{0}` holds more values than fit in 32 bits")]
    TooWide(String),
    #[error("level `{0}` is written as a range but has no level below it")]
    RangeWithoutChildren(String),
    #[error("level `{0}`'s separator may not contain digits, `-` or spaces")]
    Separator(String),
}

impl Scheme {
    pub fn validate(&self) -> Result<(), SchemeError> {
        if !(2..=MAX_BASE).contains(&self.base) {
            return Err(SchemeError::Base(self.base));
        }
        if self.levels.is_empty() {
            return Err(SchemeError::NoLevels);
        }
        for (i, level) in self.levels.iter().enumerate() {
            if level.width == 0 {
                return Err(SchemeError::Width(level.name.clone()));
            }
            if self.base.checked_pow(level.width).is_none() {
                return Err(SchemeError::TooWide(level.name.clone()));
            }
            if level.range && i + 1 == self.levels.len() {
                return Err(SchemeError::RangeWithoutChildren(level.name.clone()));
            }
            // Parsing relies on these never appearing in a separator.
            if level
                .separator
                .chars()
                .any(|c| c == '-' || c.is_whitespace() || self.digit_value(c).is_some())
            {
                return Err(SchemeError::Separator(level.name.clone()));
            }
        }
        Ok(())
    }

    /// The number of values a level can hold: `base ^ width`.
    pub fn capacity(&self, level: usize) -> u32 {
        self.base.pow(self.levels[level].width)
    }

    pub(crate) fn digit_value(&self, c: char) -> Option<u32> {
        let v = c.to_digit(MAX_BASE)?;
        (v < self.base).then_some(v)
    }

    pub(crate) fn write_digits(&self, out: &mut String, mut value: u32, width: u32) {
        let start = out.len();
        for _ in 0..width {
            out.insert(start, ALPHABET[(value % self.base) as usize] as char);
            value /= self.base;
        }
    }
}
