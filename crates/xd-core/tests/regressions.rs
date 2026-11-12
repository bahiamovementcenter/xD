//! Bugs found by hand or by proptest, pinned by name (docs/testing.md,
//! principle 5).

use xd_core::{
    Address, AddressError, Entry, Level, Policy, Registry, RegistryError, Scheme, SchemeError,
    alphabets, jd, name,
};

/// Base 64 with three 5-digit levels: each level fits in 32 bits, but a whole
/// address needs 90, so `Address::to_integer` used to overflow.
#[test]
fn schemes_whose_addresses_need_more_than_64_bits_are_rejected() {
    let level = |n: &str| Level {
        name: n.into(),
        width: 5,
        separator: ".".into(),
        range: false,
    };
    let mut s = Scheme {
        base: 64,
        alphabet: Some(alphabets::BASE64URL.into()),
        range_separator: "~".into(),
        levels: vec![level("a"), level("b"), level("c")],
    };
    assert_eq!(s.validate(), Err(SchemeError::TooLarge));

    // Two levels (60 bits) still fit, and the largest address converts.
    s.levels.pop();
    s.validate().unwrap();
    let max = s.capacity(0) - 1;
    let a = Address::new(&s, vec![max, max]).unwrap();
    assert_eq!(a.to_integer(&s), Ok((1u64 << 60) - 1));
}

/// An address doesn't carry its scheme. Rendering a JD ID with a 2-level
/// scheme used to index past the scheme's levels and panic.
#[test]
fn an_address_from_another_scheme_is_an_error_not_a_panic() {
    let jd = jd::scheme();
    let mut short = jd.clone();
    short.levels.truncate(2);
    let id = Address::new(&jd, vec![1, 1, 11]).unwrap();

    let too_deep = AddressError::TooDeep { levels: 2, got: 3 };
    assert_eq!(id.render(&short), Err(too_deep.clone()));
    assert_eq!(name::render(&short, &id, Some("x")), Err(too_deep.clone()));
    assert_eq!(id.to_integer(&short), Err(too_deep));

    // A value too large for the other scheme's level, too.
    let mut binary = jd.clone();
    binary.base = 2;
    assert!(matches!(
        id.render(&binary),
        Err(AddressError::OutOfRange { .. })
    ));
}

#[test]
fn allocating_under_a_parent_from_another_scheme_is_invalid() {
    let jd = jd::scheme();
    let mut short = jd.clone();
    short.levels.truncate(2);
    let mut r = Registry::new();
    for v in [vec![1], vec![1, 1], vec![1, 1, 11]] {
        r.insert(
            &jd,
            Entry {
                address: Address::new(&jd, v).unwrap(),
                label: "x".into(),
            },
        )
        .unwrap();
    }
    let id = Address::new(&jd, vec![1, 1, 11]).unwrap();
    assert_eq!(
        r.allocate(&short, Some(&id), Policy::AfterHighest, &jd::standard_zeros),
        Err(RegistryError::Invalid(id))
    );
}
