//! The laws, over generated schemes: bases 2 to 64, default and custom
//! alphabets, one to five levels, any widths that fit in 64 bits.

use proptest::prelude::*;
use xd_conformance::{cases, laws, strategies};
use xd_core::name;
use xd_core::{Address, jd};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases(512)))]

    #[test]
    fn put_get((s, a) in strategies::scheme_and_address()) {
        laws::put_get(&s, &a)?;
    }

    #[test]
    fn get_put_name((s, a) in strategies::scheme_and_address(), label in strategies::label()) {
        laws::get_put_name(&s, &a, &label)?;
    }

    #[test]
    fn sibling_order((s, a, b) in strategies::scheme_and_siblings()) {
        laws::sibling_order(&s, &a, &b)?;
    }

    #[test]
    fn case_safe_unless_reported((s, a, b) in strategies::scheme_and_siblings()) {
        laws::case_safe_unless_reported(&s, &a, &b)?;
    }

    // Number-like text reaches the parser's branches; anything it accepts
    // must be canonical, and nothing may panic.
    #[test]
    fn parse_accepts_only_canonical(
        (s, input) in strategies::scheme().prop_flat_map(|s| (Just(s.clone()), strategies::number_like(&s)))
    ) {
        laws::parse_accepts_only_canonical(&s, &input)?;
        let _ = name::parse(&s, &input);
    }

    #[test]
    fn near_misses_are_rejected_or_canonical(
        (s, input) in strategies::scheme().prop_flat_map(|s| (Just(s.clone()), strategies::near_number(&s)))
    ) {
        laws::parse_accepts_only_canonical(&s, &input)?;
    }

    // Names come from disks and other people: arbitrary text never panics.
    #[test]
    fn parse_never_panics(s in strategies::scheme(), input in any::<String>()) {
        let _ = Address::parse(&s, &input);
        let _ = name::parse(&s, &input);
    }

    // Every scheme validate accepts: no operation on any address panics.
    #[test]
    fn validated_schemes_are_safe((s, a) in strategies::scheme_and_address()) {
        a.render(&s).unwrap();
        a.to_integer(&s).unwrap();
        for i in 0..s.levels.len() {
            let _ = s.capacity(i);
        }
        let _ = s.portability();
    }
}

#[test]
fn jd_passes_the_laws_on_every_address() {
    // JD is small enough to check exhaustively: 10 areas, 100 categories,
    // 10,000 IDs.
    let s = jd::scheme();
    let mut all = Vec::new();
    for area in 0..10 {
        all.push(vec![area]);
        for cat in 0..10 {
            all.push(vec![area, cat]);
            for id in 0..100 {
                all.push(vec![area, cat, id]);
            }
        }
    }
    for v in all {
        let a = Address::new(&s, v).unwrap();
        laws::put_get(&s, &a).unwrap();
        laws::get_put_name(&s, &a, "Label").unwrap();
    }
}

#[test]
fn jd_ranges_parse_only_when_exact() {
    let s = jd::scheme();
    let area = Address::new(&s, vec![1]).unwrap();
    assert_eq!(Address::parse(&s, "10-19"), Some(area));
    for bad in [
        "10-18", "10-29", "11-19", "1", "10-", "-19", "10--19", "10-19-",
    ] {
        assert_eq!(Address::parse(&s, bad), None, "{bad}");
    }
}
