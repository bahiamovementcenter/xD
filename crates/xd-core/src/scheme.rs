//! A numbering scheme: the base and the levels of an address (formal 1.1).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The default digits: above 9 they are letters, read in either case, so a
/// base up to 36 is safe on case-insensitive disks (experiments E1). Larger
/// bases need an [`Scheme::alphabet`].
const DEFAULT_ALPHABET: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const MAX_DEFAULT_BASE: u32 = 36;

/// Ready-made alphabets for [`Scheme::alphabet`].
pub mod alphabets {
    /// Bitcoin's base58: no `0`, `O`, `I` or `l`. Mixed case.
    pub const BASE58: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    /// Digits, then upper case, then lower case: byte order is digit order.
    pub const BASE62: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    /// RFC 4648 base64url, the file-name-safe base64 (`-` and `_` in place of
    /// `+` and `/`). Its `-` needs another [`super::Scheme::range_separator`].
    pub const BASE64URL: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
}

/// Characters a file name can't hold on Windows (or anywhere, for `/`).
const UNPORTABLE: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// How numbers are built and written. Johnny.Decimal is one value of this
/// (see [`crate::jd::scheme`]); nothing in the core assumes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scheme {
    pub base: u32,
    /// The digits, lowest first, exactly `base` of them, read exactly as
    /// written (case matters). Without one, digits are `0-9A-Z`, read in
    /// either case, and the base can be at most 36.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alphabet: Option<String>,
    /// Between the first and last child of a level written as a range, as in
    /// JD's `10-19`.
    #[serde(default = "default_range_separator")]
    pub range_separator: String,
    /// From the top of the tree down. An address has one value per level,
    /// from the first level to its own.
    pub levels: Vec<Level>,
}

fn default_range_separator() -> String {
    "-".into()
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
    #[error("base {0} is below 2")]
    Base(u32),
    #[error("base {0} needs an alphabet: the default digits stop at {MAX_DEFAULT_BASE}")]
    NeedsAlphabet(u32),
    #[error("the alphabet has {got} digits, but the base is {base}")]
    AlphabetLength { base: u32, got: usize },
    #[error("the alphabet repeats `{0}`")]
    RepeatedDigit(char),
    #[error("the alphabet can't contain `{0}`: it can't appear in a file name")]
    UnusableDigit(char),
    #[error("the range separator must be non-empty and use no digits or spaces")]
    RangeSeparator,
    #[error("a scheme needs at least one level")]
    NoLevels,
    #[error("level `{0}` has width 0")]
    Width(String),
    #[error("level `{0}` holds more values than fit in 32 bits")]
    TooWide(String),
    #[error("level `{0}` is written as a range but has no level below it")]
    RangeWithoutChildren(String),
    #[error("level `{0}`'s separator may not contain digits, spaces or the range separator")]
    Separator(String),
}

/// Something that works on Linux but not on every disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Portability {
    /// Two digits differ only in case, so on macOS and Windows disks two
    /// different numbers can be the same folder.
    CaseCollision(char, char),
    /// A digit or separator Windows doesn't allow in file names.
    WindowsForbidden(char),
}

impl Scheme {
    pub fn validate(&self) -> Result<(), SchemeError> {
        if self.base < 2 {
            return Err(SchemeError::Base(self.base));
        }
        match &self.alphabet {
            None if self.base > MAX_DEFAULT_BASE => {
                return Err(SchemeError::NeedsAlphabet(self.base));
            }
            None => {}
            Some(alphabet) => {
                let digits: Vec<char> = alphabet.chars().collect();
                if digits.len() != self.base as usize {
                    return Err(SchemeError::AlphabetLength {
                        base: self.base,
                        got: digits.len(),
                    });
                }
                for (i, &c) in digits.iter().enumerate() {
                    if digits[..i].contains(&c) {
                        return Err(SchemeError::RepeatedDigit(c));
                    }
                    if c == '/' || c.is_whitespace() || c.is_control() {
                        return Err(SchemeError::UnusableDigit(c));
                    }
                }
            }
        }
        let range = &self.range_separator;
        if range.is_empty()
            || range
                .chars()
                .any(|c| c.is_whitespace() || self.digit_value(c).is_some())
        {
            return Err(SchemeError::RangeSeparator);
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
                .any(|c| c.is_whitespace() || self.digit_value(c).is_some())
                || level.separator.contains(range.as_str())
                || (!level.separator.is_empty() && range.contains(level.separator.as_str()))
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

    /// True when sorting rendered numbers as plain strings (byte order, as
    /// `sort` and `ls` do in the C locale) gives the order of the tree: the
    /// digits must be in ascending code-point order.
    pub fn sorts_by_name(&self) -> bool {
        match &self.alphabet {
            None => true,
            Some(a) => a.chars().zip(a.chars().skip(1)).all(|(x, y)| x < y),
        }
    }

    /// What would break on macOS or Windows disks. Empty for the default
    /// digits with the default separators.
    pub fn portability(&self) -> Vec<Portability> {
        let mut issues = Vec::new();
        if let Some(alphabet) = &self.alphabet {
            let digits: Vec<char> = alphabet.chars().collect();
            for (i, &a) in digits.iter().enumerate() {
                if let Some(&b) = digits[..i]
                    .iter()
                    .find(|&&b| b != a && b.to_lowercase().eq(a.to_lowercase()))
                {
                    issues.push(Portability::CaseCollision(b, a));
                }
            }
        }
        let written = self
            .alphabet
            .iter()
            .flat_map(|a| a.chars())
            .chain(self.range_separator.chars())
            .chain(self.levels.iter().flat_map(|l| l.separator.chars()));
        for c in written {
            if UNPORTABLE.contains(&c) && !issues.contains(&Portability::WindowsForbidden(c)) {
                issues.push(Portability::WindowsForbidden(c));
            }
        }
        issues
    }

    pub(crate) fn digit_value(&self, c: char) -> Option<u32> {
        match &self.alphabet {
            None => c.to_digit(MAX_DEFAULT_BASE).filter(|&v| v < self.base),
            Some(alphabet) => alphabet.chars().position(|d| d == c).map(|i| i as u32),
        }
    }

    fn digit(&self, value: u32) -> char {
        match &self.alphabet {
            None => DEFAULT_ALPHABET[value as usize] as char,
            Some(alphabet) => alphabet.chars().nth(value as usize).expect("value < base"),
        }
    }

    pub(crate) fn write_digits(&self, out: &mut String, mut value: u32, width: u32) {
        let mut digits = Vec::with_capacity(width as usize);
        for _ in 0..width {
            digits.push(self.digit(value % self.base));
            value /= self.base;
        }
        out.extend(digits.into_iter().rev());
    }
}
