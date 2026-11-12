use proptest::prelude::*;
use xd_core::name::{self, Parsed};
use xd_core::{Address, Entry, Level, Policy, Registry, RegistryError, Scheme, jd};

fn addr(values: &[u32]) -> Address {
    Address::new(&jd::scheme(), values.to_vec()).unwrap()
}

fn hex_scheme() -> Scheme {
    Scheme {
        base: 16,
        ..jd::scheme()
    }
}

#[test]
fn jd_scheme_is_valid() {
    jd::scheme().validate().unwrap();
    hex_scheme().validate().unwrap();
}

#[test]
fn renders_jd_numbers() {
    let s = jd::scheme();
    assert_eq!(addr(&[1]).render(&s), "10-19");
    assert_eq!(addr(&[0]).render(&s), "00-09");
    assert_eq!(addr(&[1, 1]).render(&s), "11");
    assert_eq!(addr(&[1, 1, 11]).render(&s), "11.11");
    assert_eq!(addr(&[1, 1, 3]).render(&s), "11.03");
}

#[test]
fn parses_jd_numbers() {
    let s = jd::scheme();
    assert_eq!(Address::parse(&s, "10-19"), Some(addr(&[1])));
    assert_eq!(Address::parse(&s, "11"), Some(addr(&[1, 1])));
    assert_eq!(Address::parse(&s, "11.11"), Some(addr(&[1, 1, 11])));
    for bad in [
        "1", "10-18", "11-19", "10-29", "111", "11.1", "11.111", "11..git", "1A", "", "-", "11.",
    ] {
        assert_eq!(Address::parse(&s, bad), None, "{bad:?}");
    }
}

#[test]
fn hierarchy_comes_from_the_digits() {
    let id = addr(&[1, 1, 11]);
    assert_eq!(id.parent(), Some(addr(&[1, 1])));
    assert_eq!(
        id.ancestors().collect::<Vec<_>>(),
        vec![addr(&[1, 1]), addr(&[1])]
    );
    assert!(addr(&[1]).is_ancestor_of(&id));
    assert!(!addr(&[1, 2]).is_ancestor_of(&id));
    assert!(!id.is_ancestor_of(&id));
}

#[test]
fn rejects_out_of_range_values() {
    let s = jd::scheme();
    assert!(Address::new(&s, vec![10]).is_err());
    assert!(Address::new(&s, vec![1, 1, 100]).is_err());
    assert!(Address::new(&s, vec![1, 1, 1, 1]).is_err());
    assert!(Address::new(&s, vec![]).is_err());
}

#[test]
fn folder_names() {
    let s = jd::scheme();
    assert_eq!(
        name::render(&s, &addr(&[1, 1, 11]), Some("Passport")),
        "11.11 Passport"
    );
    assert_eq!(
        name::parse(&s, "11.11 Passport"),
        Parsed::Numbered {
            address: addr(&[1, 1, 11]),
            label: Some("Passport".into())
        }
    );
    assert_eq!(
        name::parse(&s, "10-19 Life admin"),
        Parsed::Numbered {
            address: addr(&[1]),
            label: Some("Life admin".into())
        }
    );
    // Outside the number space, so extensions can't collide with numbers.
    for skip in [
        "+ Wendy",
        ".jd.ncl",
        "Notes",
        "11.11Passport",
        "W0011~11.13 Audit",
    ] {
        assert_eq!(name::parse(&s, skip), Parsed::Skip, "{skip:?}");
    }
}

#[test]
fn hex_base_changes_only_the_digits() {
    let s = hex_scheme();
    let a = Address::new(&s, vec![1]).unwrap();
    assert_eq!(a.render(&s), "10-1F");
    let id = Address::new(&s, vec![1, 10, 0x3F]).unwrap();
    assert_eq!(id.render(&s), "1A.3F");
    assert_eq!(Address::parse(&s, "1a.3f"), Some(id));
}

#[test]
fn integer_form_keeps_tree_order() {
    let s = jd::scheme();
    assert_eq!(addr(&[1, 1, 13]).to_integer(&s), 1113);
    assert_eq!(addr(&[9, 9, 99]).to_integer(&s), 9999);
}

#[test]
fn invalid_schemes_are_rejected() {
    let mut s = jd::scheme();
    s.base = 37;
    assert!(s.validate().is_err());
    let mut s = jd::scheme();
    s.levels[2].range = true;
    assert!(s.validate().is_err());
    let mut s = jd::scheme();
    s.levels[2].separator = "-".into();
    assert!(s.validate().is_err());
    let s = Scheme {
        base: 10,
        alphabet: None,
        range_separator: "-".into(),
        levels: vec![Level {
            name: "x".into(),
            width: 0,
            separator: String::new(),
            range: false,
        }],
    };
    assert!(s.validate().is_err());
}

fn system() -> Registry {
    let s = jd::scheme();
    let mut r = Registry::new();
    for (a, label) in [
        (vec![1], "Life admin"),
        (vec![1, 1], "Me"),
        (vec![1, 1, 0], "JDex"),
        (vec![1, 1, 11], "Passport"),
    ] {
        r.insert(
            &s,
            Entry {
                address: addr(&a),
                label: label.into(),
            },
        )
        .unwrap();
    }
    r
}

#[test]
fn allocation_follows_jd() {
    let s = jd::scheme();
    let mut r = system();
    let cat = addr(&[1, 1]);
    let next = |r: &Registry| {
        r.allocate(&s, Some(&cat), Policy::AfterHighest, &jd::standard_zeros)
            .unwrap()
    };
    assert_eq!(next(&r), addr(&[1, 1, 12]));

    // A retired number is never handed out again, even as the highest.
    r.insert(
        &s,
        Entry {
            address: addr(&[1, 1, 12]),
            label: "Old".into(),
        },
    )
    .unwrap();
    r.retire(&addr(&[1, 1, 12])).unwrap();
    assert_eq!(next(&r), addr(&[1, 1, 13]));

    // In an empty category the first regular ID is .11.
    r.insert(
        &s,
        Entry {
            address: addr(&[1, 2]),
            label: "Home".into(),
        },
    )
    .unwrap();
    assert_eq!(
        r.allocate(
            &s,
            Some(&addr(&[1, 2])),
            Policy::AfterHighest,
            &jd::standard_zeros
        )
        .unwrap(),
        addr(&[1, 2, 11])
    );
    // Categories skip the area's own x0.
    assert_eq!(
        r.allocate(
            &s,
            Some(&addr(&[1])),
            Policy::AfterHighest,
            &jd::standard_zeros
        )
        .unwrap(),
        addr(&[1, 3])
    );
}

#[test]
fn lowest_free_fills_gaps_but_not_retired_numbers() {
    let s = jd::scheme();
    let mut r = system();
    for id in [12, 14] {
        r.insert(
            &s,
            Entry {
                address: addr(&[1, 1, id]),
                label: "x".into(),
            },
        )
        .unwrap();
    }
    r.retire(&addr(&[1, 1, 12])).unwrap();
    assert_eq!(
        r.allocate(
            &s,
            Some(&addr(&[1, 1])),
            Policy::LowestFree,
            &jd::standard_zeros
        )
        .unwrap(),
        addr(&[1, 1, 13])
    );
}

#[test]
fn a_full_category_says_so() {
    let s = jd::scheme();
    let mut r = system();
    r.insert(
        &s,
        Entry {
            address: addr(&[1, 1, 99]),
            label: "Last".into(),
        },
    )
    .unwrap();
    assert!(matches!(
        r.allocate(&s, Some(&addr(&[1, 1])), Policy::AfterHighest, &jd::standard_zeros),
        Err(RegistryError::Full(c)) if c == "11"
    ));
}

#[test]
fn registry_invariants() {
    let s = jd::scheme();
    let mut r = system();
    let passport = Entry {
        address: addr(&[1, 1, 11]),
        label: "Passport".into(),
    };
    assert_eq!(
        r.insert(&s, passport.clone()),
        Err(RegistryError::Duplicate(addr(&[1, 1, 11])))
    );
    assert_eq!(
        r.insert(
            &s,
            Entry {
                address: addr(&[2, 1, 11]),
                label: "Orphan".into()
            }
        ),
        Err(RegistryError::MissingParent(addr(&[2, 1, 11])))
    );
    assert_eq!(
        r.retire(&addr(&[1, 1])),
        Err(RegistryError::HasLiveChildren(addr(&[1, 1])))
    );
    r.rename(&addr(&[1, 1, 11]), "Travel documents".into())
        .unwrap();
    assert_eq!(r.get(&addr(&[1, 1, 11])).unwrap().label, "Travel documents");
    r.retire(&addr(&[1, 1, 11])).unwrap();
    assert_eq!(
        r.insert(&s, passport),
        Err(RegistryError::Reused(addr(&[1, 1, 11])))
    );
}

fn any_scheme() -> impl Strategy<Value = Scheme> {
    (2u32..=36, 1u32..=3, 1u32..=2, any::<bool>()).prop_map(|(base, w1, w2, range)| Scheme {
        base,
        alphabet: None,
        range_separator: "-".into(),
        levels: vec![
            Level {
                name: "a".into(),
                width: w1,
                separator: String::new(),
                range,
            },
            Level {
                name: "b".into(),
                width: 1,
                separator: String::new(),
                range: false,
            },
            Level {
                name: "c".into(),
                width: w2,
                separator: ".".into(),
                range: false,
            },
        ],
    })
}

fn scheme_and_address() -> impl Strategy<Value = (Scheme, Address)> {
    any_scheme().prop_flat_map(|s| {
        let caps: Vec<u32> = (0..s.levels.len()).map(|i| s.capacity(i)).collect();
        (1..=s.levels.len())
            .prop_flat_map(move |depth| caps[..depth].iter().map(|&c| 0..c).collect::<Vec<_>>())
            .prop_map(move |values| {
                let a = Address::new(&s, values).unwrap();
                (s.clone(), a)
            })
    })
}

proptest! {
    // Formal 1.5: parse(render(e)) = e.n, for any scheme, not only JD's.
    #[test]
    fn render_then_parse_round_trips((s, a) in scheme_and_address(), label in "[A-Za-z][A-Za-z ]{0,10}") {
        prop_assert_eq!(Address::parse(&s, &a.render(&s)), Some(a.clone()));
        prop_assert_eq!(
            name::parse(&s, &name::render(&s, &a, Some(&label))),
            Parsed::Numbered { address: a, label: Some(label) }
        );
    }

    // Sorting rendered names of one level as plain strings sorts the tree,
    // for bases up to 36 (experiments E1: plain `sort` is right).
    #[test]
    fn plain_string_order_is_tree_order((s, a) in scheme_and_address(), (_, b) in scheme_and_address()) {
        // `b` comes from another scheme; keep it only if it fits this one.
        let Ok(b) = Address::new(&s, b.values().to_vec()) else { return Ok(()) };
        prop_assume!(a.level() == b.level());
        prop_assert_eq!(a.render(&s).cmp(&b.render(&s)), a.cmp(&b));
    }
}
