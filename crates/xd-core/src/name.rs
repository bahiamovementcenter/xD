//! The folder-name projection (formal 1.5): `11.11 Passport` and back.

use crate::address::{Address, AddressError};
use crate::scheme::Scheme;

/// What a name means to the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// A number, with the label after the first space if there is one.
    Numbered {
        address: Address,
        label: Option<String>,
    },
    /// Outside the number space. Extensions (`+` children, dot entries,
    /// ordinary subfolders) live here without colliding with numbers.
    Skip,
}

/// Before the first space is identity; after it is a label you can change.
/// Fails only for an address that doesn't fit `scheme`.
pub fn render(
    scheme: &Scheme,
    address: &Address,
    label: Option<&str>,
) -> Result<String, AddressError> {
    let number = address.render(scheme)?;
    Ok(match label {
        Some(label) if !label.is_empty() => format!("{number} {label}"),
        _ => number,
    })
}

/// Accepts the number alone or the number, a space and a label. An empty
/// label (`"11.11 "`) is treated as no label.
pub fn parse(scheme: &Scheme, name: &str) -> Parsed {
    let (number, label) = match name.split_once(' ') {
        Some((number, label)) => (number, (!label.is_empty()).then(|| label.to_string())),
        None => (name, None),
    };
    match Address::parse(scheme, number) {
        Some(address) => Parsed::Numbered { address, label },
        None => Parsed::Skip,
    }
}
