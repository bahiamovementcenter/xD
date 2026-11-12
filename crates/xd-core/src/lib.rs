//! The core of xD (Extensible Decimal), following `docs/jd-formal.md`
//! section 1: the address space and hierarchy ([`address`]), entries,
//! allocation and retirement ([`registry`]) and the folder-name projection
//! ([`name`]), all generic over a [`scheme::Scheme`]. Johnny.Decimal is one
//! scheme plus modules ([`jd`]).

pub mod address;
pub mod jd;
pub mod name;
pub mod registry;
pub mod scheme;

pub use address::{Address, AddressError};
pub use registry::{Entry, Policy, Registry, RegistryError, Reserved};
pub use scheme::{Level, Scheme, SchemeError};
