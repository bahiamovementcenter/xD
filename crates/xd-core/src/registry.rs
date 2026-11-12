//! Entries, allocation and retirement (formal 1.3, 1.4).

use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

use crate::address::Address;
use crate::scheme::Scheme;

/// An entry: the number is its identity, the label is a name you can change
/// (formal 1.3). Renaming is changing the label of the same entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub address: Address,
    pub label: String,
}

/// Which free number [`Registry::allocate`] picks. The invariant (never a
/// live, retired or reserved number) holds for every policy; the choice
/// between free numbers is policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Policy {
    /// One more than the highest number ever used under the parent, live or
    /// retired: JD's "in the sequence that you created them".
    #[default]
    AfterHighest,
    /// The lowest free number, for schemes that want gaps filled.
    LowestFree,
}

/// Numbers that allocation skips, such as JD's standard zeros (formal 2.1).
/// They can still be added by hand with [`Registry::insert`].
pub trait Reserved {
    fn is_reserved(&self, address: &Address) -> bool;
}

/// No reserved numbers.
pub struct NoneReserved;

impl Reserved for NoneReserved {
    fn is_reserved(&self, _: &Address) -> bool {
        false
    }
}

impl<F: Fn(&Address) -> bool> Reserved for F {
    fn is_reserved(&self, address: &Address) -> bool {
        self(address)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegistryError {
    #[error("{0} is already in use")]
    Duplicate(Address),
    #[error("{0} was retired; numbers are never reused")]
    Reused(Address),
    #[error("{0} has no live parent")]
    MissingParent(Address),
    #[error("{0} doesn't fit the scheme")]
    Invalid(Address),
    #[error("{0} isn't live")]
    NotLive(Address),
    #[error("{0} still has live children")]
    HasLiveChildren(Address),
    #[error("{0} has no free number left; it may be too broad and need splitting")]
    Full(String),
}

/// The live and retired numbers of one system. `live` and `retired` never
/// overlap, and `retired` only grows.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    live: BTreeMap<Address, String>,
    retired: BTreeSet<Address>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, address: &Address) -> Option<Entry> {
        self.live.get(address).map(|label| Entry {
            address: address.clone(),
            label: label.clone(),
        })
    }

    pub fn is_live(&self, address: &Address) -> bool {
        self.live.contains_key(address)
    }

    pub fn is_retired(&self, address: &Address) -> bool {
        self.retired.contains(address)
    }

    /// Live entries in tree order.
    pub fn entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.live.iter().map(|(address, label)| Entry {
            address: address.clone(),
            label: label.clone(),
        })
    }

    pub fn retired(&self) -> impl Iterator<Item = &Address> {
        self.retired.iter()
    }

    /// Adds an entry with a number chosen by the caller.
    pub fn insert(&mut self, scheme: &Scheme, entry: Entry) -> Result<(), RegistryError> {
        let address = entry.address;
        if Address::new(scheme, address.values().to_vec()).is_err() {
            return Err(RegistryError::Invalid(address));
        }
        if self.live.contains_key(&address) {
            return Err(RegistryError::Duplicate(address));
        }
        if self.retired.contains(&address) {
            return Err(RegistryError::Reused(address));
        }
        if let Some(parent) = address.parent()
            && !self.live.contains_key(&parent)
        {
            return Err(RegistryError::MissingParent(address));
        }
        self.live.insert(address, entry.label);
        Ok(())
    }

    /// Changes a label. The number, and so the identity, stays.
    pub fn rename(&mut self, address: &Address, label: String) -> Result<(), RegistryError> {
        match self.live.get_mut(address) {
            Some(old) => {
                *old = label;
                Ok(())
            }
            None => Err(RegistryError::NotLive(address.clone())),
        }
    }

    /// Moves a number from live to retired, for good. Where the retired
    /// thing is stored (an archive strategy, formal 2.5) isn't decided here.
    pub fn retire(&mut self, address: &Address) -> Result<(), RegistryError> {
        if !self.live.contains_key(address) {
            return Err(RegistryError::NotLive(address.clone()));
        }
        if self.live.keys().any(|a| address.is_ancestor_of(a)) {
            return Err(RegistryError::HasLiveChildren(address.clone()));
        }
        self.live.remove(address);
        self.retired.insert(address.clone());
        Ok(())
    }

    /// The next number under `parent` (or at the top level for `None`),
    /// skipping live, retired and reserved numbers. It isn't added: pass it
    /// to [`Registry::insert`] with a label.
    pub fn allocate(
        &self,
        scheme: &Scheme,
        parent: Option<&Address>,
        policy: Policy,
        reserved: &dyn Reserved,
    ) -> Result<Address, RegistryError> {
        let level = parent.map_or(0, |p| p.level() + 1);
        let full = || {
            RegistryError::Full(
                parent.map_or_else(|| "the top level".to_string(), |p| p.render(scheme)),
            )
        };
        if level >= scheme.levels.len() {
            return Err(full());
        }
        if let Some(p) = parent
            && !self.live.contains_key(p)
        {
            return Err(RegistryError::NotLive(p.clone()));
        }
        let child = |value| match parent {
            Some(p) => p.child_unchecked(value),
            None => Address::root_unchecked(value),
        };
        let used = |a: &Address| self.live.contains_key(a) || self.retired.contains(a);
        let free = |a: &Address| !used(a) && !reserved.is_reserved(a);
        let capacity = scheme.capacity(level);
        let start = match policy {
            Policy::LowestFree => 0,
            Policy::AfterHighest => (0..capacity)
                .rev()
                .find(|&v| used(&child(v)))
                .map_or(0, |v| v + 1),
        };
        (start..capacity)
            .map(child)
            .find(|a| free(a))
            .ok_or_else(full)
    }
}
