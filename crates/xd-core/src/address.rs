//! Addresses, the hierarchy (formal 1.1, 1.2) and how a number is written.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

use crate::scheme::Scheme;

/// One value per level, from the top. Its length is its level, and its
/// parent is computed from its digits (formal 1.2), so an address can't move
/// to another parent without becoming a different address.
///
/// The derived order is the prefix order with siblings in numeric order,
/// which is the order of the tree.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Address(Vec<u32>);

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AddressError {
    #[error("an address needs at least one level")]
    Empty,
    #[error("the scheme has {levels} levels, but the address has {got}")]
    TooDeep { levels: usize, got: usize },
    #[error("{value} doesn't fit level `{level}`, which holds {capacity} values")]
    OutOfRange {
        level: String,
        value: u32,
        capacity: u32,
    },
}

impl Address {
    pub fn new(scheme: &Scheme, values: Vec<u32>) -> Result<Self, AddressError> {
        if values.is_empty() {
            return Err(AddressError::Empty);
        }
        if values.len() > scheme.levels.len() {
            return Err(AddressError::TooDeep {
                levels: scheme.levels.len(),
                got: values.len(),
            });
        }
        for (i, &value) in values.iter().enumerate() {
            let capacity = scheme.capacity(i);
            if value >= capacity {
                return Err(AddressError::OutOfRange {
                    level: scheme.levels[i].name.clone(),
                    value,
                    capacity,
                });
            }
        }
        Ok(Address(values))
    }

    pub fn values(&self) -> &[u32] {
        &self.0
    }

    /// The index of this address's level in the scheme.
    pub fn level(&self) -> usize {
        self.0.len() - 1
    }

    pub fn last(&self) -> u32 {
        *self.0.last().expect("addresses are never empty")
    }

    pub fn parent(&self) -> Option<Address> {
        (self.0.len() > 1).then(|| Address(self.0[..self.0.len() - 1].to_vec()))
    }

    /// Every ancestor, nearest first.
    pub fn ancestors(&self) -> impl Iterator<Item = Address> + '_ {
        (1..self.0.len())
            .rev()
            .map(|n| Address(self.0[..n].to_vec()))
    }

    pub fn is_ancestor_of(&self, other: &Address) -> bool {
        self.0.len() < other.0.len() && other.0.starts_with(&self.0)
    }

    /// The child with `value` at the next level, unchecked against a scheme.
    pub(crate) fn child_unchecked(&self, value: u32) -> Address {
        let mut values = self.0.clone();
        values.push(value);
        Address(values)
    }

    pub(crate) fn root_unchecked(value: u32) -> Address {
        Address(vec![value])
    }

    /// The number as the scheme writes it: `10-19`, `11`, `11.11`.
    pub fn render(&self, scheme: &Scheme) -> String {
        if scheme.levels[self.level()].range {
            let max = scheme.capacity(self.level() + 1) - 1;
            format!(
                "{}-{}",
                self.child_unchecked(0).render_digits(scheme),
                self.child_unchecked(max).render_digits(scheme)
            )
        } else {
            self.render_digits(scheme)
        }
    }

    fn render_digits(&self, scheme: &Scheme) -> String {
        let mut out = String::new();
        for (level, &value) in scheme.levels.iter().zip(&self.0) {
            out.push_str(&level.separator);
            scheme.write_digits(&mut out, value, level.width);
        }
        out
    }

    /// The inverse of [`Address::render`]. Only the number: a label after it
    /// is the job of [`crate::name`]. Letters are read in either case.
    pub fn parse(scheme: &Scheme, s: &str) -> Option<Address> {
        // A separator never contains `-` (Scheme::validate), so a `-` always
        // splits a range.
        if let Some((lo, hi)) = s.split_once('-') {
            let lo = Self::parse_digits(scheme, lo)?;
            let hi = Self::parse_digits(scheme, hi)?;
            let parent = lo.parent()?;
            let max = scheme.capacity(lo.level()) - 1;
            let ok = hi.parent().as_ref() == Some(&parent)
                && lo.last() == 0
                && hi.last() == max
                && scheme.levels[parent.level()].range;
            return ok.then_some(parent);
        }
        let address = Self::parse_digits(scheme, s)?;
        // A level written as a range is never written as its bare digits.
        (!scheme.levels[address.level()].range).then_some(address)
    }

    fn parse_digits(scheme: &Scheme, s: &str) -> Option<Address> {
        let mut rest = s;
        let mut values = Vec::new();
        for level in &scheme.levels {
            if rest.is_empty() && !values.is_empty() {
                break;
            }
            rest = rest.strip_prefix(level.separator.as_str())?;
            let mut value = 0u32;
            let mut chars = rest.chars();
            for _ in 0..level.width {
                value = value * scheme.base + scheme.digit_value(chars.next()?)?;
            }
            rest = chars.as_str();
            values.push(value);
        }
        (rest.is_empty() && !values.is_empty()).then_some(Address(values))
    }

    /// Integer form (experiments E2): all levels as one number in the base,
    /// so sorting integers of one level sorts the names.
    pub fn to_integer(&self, scheme: &Scheme) -> u64 {
        scheme
            .levels
            .iter()
            .zip(&self.0)
            .fold(0u64, |acc, (level, &v)| {
                acc * u64::from(scheme.base).pow(level.width) + u64::from(v)
            })
    }
}

/// Writes the raw values; use [`Address::render`] for the scheme's form.
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.0.iter().map(u32::to_string).collect();
        write!(f, "[{}]", parts.join(", "))
    }
}
