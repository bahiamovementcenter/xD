//! The registry (formal 1.3, 1.4) against a model: random sequences of
//! insert, rename, retire and allocate, on small schemes so sequences reach
//! full levels and deep trees. After every step the registry must agree
//! with a deliberately naive model and keep the invariants:
//!
//! - active and retired never overlap, and retired only grows;
//! - every active entry's parent is active;
//! - allocate never returns an active, retired or reserved number, and
//!   `AfterHighest` never returns one below a number already used there.

use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use xd_conformance::cases;
use xd_core::{Address, Entry, Level, Policy, Registry, RegistryError, Scheme};

fn small_scheme() -> impl Strategy<Value = Scheme> {
    (2u32..=4, prop::collection::vec(1u32..=2, 1..=3)).prop_map(|(base, widths)| Scheme {
        base,
        alphabet: None,
        range_separator: "-".into(),
        levels: widths
            .into_iter()
            .enumerate()
            .map(|(i, width)| Level {
                name: format!("l{i}"),
                width,
                separator: if i == 0 { String::new() } else { ".".into() },
                range: false,
            })
            .collect(),
    })
}

#[derive(Debug, Clone)]
enum Op {
    Insert(Vec<u32>),
    Rename(Vec<u32>),
    Retire(Vec<u32>),
    /// Allocate under the given prefix (empty = the top level), then insert
    /// what came back.
    Allocate(Vec<u32>, Policy),
}

/// Values may go one past a level's capacity, so invalid numbers are tried.
fn values(s: &Scheme) -> impl Strategy<Value = Vec<u32>> + use<> {
    let caps: Vec<u32> = (0..s.levels.len()).map(|i| s.capacity(i)).collect();
    (1..=caps.len() + 1).prop_flat_map(move |depth| {
        (0..depth)
            .map(|i| 0..=caps.get(i).copied().unwrap_or(1))
            .collect::<Vec<_>>()
    })
}

fn op(s: &Scheme) -> impl Strategy<Value = Op> + use<> {
    let policy = prop_oneof![Just(Policy::AfterHighest), Just(Policy::LowestFree)];
    prop_oneof![
        4 => values(s).prop_map(Op::Insert),
        1 => values(s).prop_map(Op::Rename),
        2 => values(s).prop_map(Op::Retire),
        // Allocation prefixes are shorter: the top level or a valid parent.
        4 => (values(s), policy).prop_map(|(mut v, p)| {
            v.pop();
            Op::Allocate(v, p)
        }),
    ]
}

/// The obvious, slow implementation.
#[derive(Default)]
struct Model {
    active: BTreeMap<Vec<u32>, String>,
    retired: BTreeSet<Vec<u32>>,
}

impl Model {
    fn fits(s: &Scheme, v: &[u32]) -> bool {
        !v.is_empty()
            && v.len() <= s.levels.len()
            && v.iter().enumerate().all(|(i, &x)| x < s.capacity(i))
    }

    fn used(&self, v: &[u32]) -> bool {
        self.active.contains_key(v) || self.retired.contains(v)
    }

    fn insert(&mut self, s: &Scheme, v: &[u32], label: &str) -> Result<(), &'static str> {
        if !Self::fits(s, v) {
            return Err("invalid");
        }
        if self.active.contains_key(v) {
            return Err("duplicate");
        }
        if self.retired.contains(v) {
            return Err("reused");
        }
        if v.len() > 1 && !self.active.contains_key(&v[..v.len() - 1]) {
            return Err("missing parent");
        }
        self.active.insert(v.to_vec(), label.to_string());
        Ok(())
    }

    fn retire(&mut self, v: &[u32]) -> Result<(), &'static str> {
        if !self.active.contains_key(v) {
            return Err("not live");
        }
        if self
            .active
            .keys()
            .any(|k| k.len() > v.len() && k.starts_with(v))
        {
            return Err("has children");
        }
        self.active.remove(v);
        self.retired.insert(v.to_vec());
        Ok(())
    }

    fn allocate(
        &self,
        s: &Scheme,
        prefix: &[u32],
        policy: Policy,
        reserved: impl Fn(&[u32]) -> bool,
    ) -> Result<Vec<u32>, &'static str> {
        if !prefix.is_empty() && !Self::fits(s, prefix) {
            return Err("invalid");
        }
        let level = prefix.len();
        if level >= s.levels.len() {
            return Err("full");
        }
        if !prefix.is_empty() && !self.active.contains_key(prefix) {
            return Err("not live");
        }
        let child = |x: u32| [prefix, &[x]].concat();
        let all: Vec<u32> = (0..s.capacity(level)).collect();
        let start = match policy {
            Policy::LowestFree => 0,
            Policy::AfterHighest => all
                .iter()
                .filter(|&&x| self.used(&child(x)))
                .max()
                .map_or(0, |x| x + 1),
        };
        all.into_iter()
            .filter(|&x| x >= start)
            .map(child)
            .find(|c| !self.used(c) && !reserved(c))
            .ok_or("full")
    }
}

fn kind(e: &RegistryError) -> &'static str {
    match e {
        RegistryError::Invalid(_) => "invalid",
        RegistryError::Duplicate(_) => "duplicate",
        RegistryError::Reused(_) => "reused",
        RegistryError::MissingParent(_) => "missing parent",
        RegistryError::NotLive(_) => "not live",
        RegistryError::HasActiveChildren(_) => "has children",
        RegistryError::Full(_) => "full",
    }
}

/// An address the registry can be handed even when it doesn't fit the
/// scheme: built against a scheme wide enough for any generated value.
fn raw(s: &Scheme, v: &[u32]) -> Address {
    let wide = Scheme {
        base: 16,
        levels: (0..v.len().max(s.levels.len()))
            .map(|i| Level {
                name: format!("w{i}"),
                width: 2,
                separator: ".".into(),
                range: false,
            })
            .collect(),
        ..s.clone()
    };
    Address::new(&wide, v.to_vec()).expect("generated values fit the wide scheme")
}

fn check_invariants(s: &Scheme, r: &Registry, m: &Model, retired_before: &BTreeSet<Vec<u32>>) {
    let active: BTreeMap<Vec<u32>, String> = r
        .entries()
        .map(|e| (e.address.values().to_vec(), e.label))
        .collect();
    let retired: BTreeSet<Vec<u32>> = r.retired().map(|a| a.values().to_vec()).collect();
    assert_eq!(active, m.active, "active entries differ from the model");
    assert_eq!(retired, m.retired, "retired numbers differ from the model");
    assert!(
        active.keys().all(|k| !retired.contains(k)),
        "active ∩ retired"
    );
    assert!(
        retired_before.is_subset(&retired),
        "a retired number came back"
    );
    for k in active.keys() {
        assert!(Model::fits(s, k), "an invalid number is active: {k:?}");
        if k.len() > 1 {
            assert!(active.contains_key(&k[..k.len() - 1]), "orphan {k:?}");
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases(256)))]

    #[test]
    fn registry_matches_the_model(
        (s, ops, reserve_zero) in small_scheme().prop_flat_map(|s| {
            (Just(s.clone()), prop::collection::vec(op(&s), 1..120), any::<bool>())
        })
    ) {
        // Reserving 0 at every level stands in for JD's standard zeros.
        let reserved = |a: &Address| reserve_zero && a.last() == 0;
        let reserved_v = |v: &[u32]| reserve_zero && v.last() == Some(&0);
        let mut r = Registry::new();
        let mut m = Model::default();
        for (step, op) in ops.into_iter().enumerate() {
            let retired_before = m.retired.clone();
            let label = format!("step {step}");
            match &op {
                Op::Insert(v) => {
                    let got = r.insert(&s, Entry { address: raw(&s, v), label: label.clone() });
                    prop_assert_eq!(got.map_err(|e| kind(&e)), m.insert(&s, v, &label), "{:?}", op);
                }
                Op::Rename(v) => {
                    let got = r.rename(&raw(&s, v), label.clone());
                    let want = match m.active.get_mut(v) {
                        Some(l) => { *l = label; Ok(()) }
                        None => Err("not live"),
                    };
                    prop_assert_eq!(got.map_err(|e| kind(&e)), want, "{:?}", op);
                }
                Op::Retire(v) => {
                    let got = r.retire(&raw(&s, v));
                    prop_assert_eq!(got.map_err(|e| kind(&e)), m.retire(v), "{:?}", op);
                }
                Op::Allocate(prefix, policy) => {
                    let parent = (!prefix.is_empty()).then(|| raw(&s, prefix));
                    let got = r.allocate(&s, parent.as_ref(), *policy, &reserved);
                    let want = m.allocate(&s, prefix, *policy, reserved_v);
                    prop_assert_eq!(
                        got.as_ref().map(|a| a.values().to_vec()).map_err(kind),
                        want.clone(),
                        "{:?}", op
                    );
                    if let Ok(a) = got {
                        prop_assert!(!m.used(a.values()) && !reserved(&a));
                        // What allocate hands out can always be inserted.
                        r.insert(&s, Entry { address: a.clone(), label: label.clone() }).unwrap();
                        m.insert(&s, a.values(), &label).unwrap();
                    }
                }
            }
            check_invariants(&s, &r, &m, &retired_before);
        }
    }

    // With nothing retired, allocating until full hands out every
    // non-reserved number of the level exactly once.
    #[test]
    fn allocation_fills_a_level_exactly(
        s in small_scheme(),
        policy in prop_oneof![Just(Policy::AfterHighest), Just(Policy::LowestFree)],
        reserve_zero in any::<bool>(),
    ) {
        let reserved = |a: &Address| reserve_zero && a.last() == 0;
        let mut r = Registry::new();
        let mut got = BTreeSet::new();
        loop {
            match r.allocate(&s, None, policy, &reserved) {
                Ok(a) => {
                    prop_assert!(got.insert(a.clone()), "{} handed out twice", a);
                    r.insert(&s, Entry { address: a, label: String::new() }).unwrap();
                }
                Err(RegistryError::Full(_)) => break,
                Err(e) => prop_assert!(false, "{}", e),
            }
        }
        let want: BTreeSet<Address> = (0..s.capacity(0))
            .map(|v| Address::new(&s, vec![v]).unwrap())
            .filter(|a| !reserved(a))
            .collect();
        prop_assert_eq!(got, want);
    }
}

#[test]
fn jd_areas_are_allocated_after_the_system_area() {
    let s = xd_core::jd::scheme();
    let r = Registry::new();
    let first = r
        .allocate(&s, None, Policy::AfterHighest, &xd_core::jd::standard_zeros)
        .unwrap();
    assert_eq!(first.render(&s).unwrap(), "10-19");
}
