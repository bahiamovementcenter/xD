//! Real folders on disk, for every base from 2 to 64: names written by the
//! scheme must come back from `read_dir` as the same numbers.

use proptest::prelude::*;
use std::collections::BTreeSet;
use std::fs;
use xd_core::name::{self, Parsed};
use xd_core::{Address, Level, Portability, Scheme, alphabets, jd};

fn levels() -> Vec<Level> {
    jd::scheme().levels
}

/// Every base from 2 to 64, from the default digits (up to 36) or the first
/// `base` characters of a larger alphabet.
fn scheme_for(base: u32) -> Scheme {
    let (alphabet, range_separator) = if base <= 36 {
        (None, "-")
    } else if base <= 62 {
        (Some(alphabets::BASE62[..base as usize].to_string()), "-")
    } else {
        // base64url uses `-` as a digit, so ranges need another separator.
        (Some(alphabets::BASE64URL[..base as usize].to_string()), "~")
    };
    Scheme {
        base,
        alphabet,
        range_separator: range_separator.into(),
        levels: levels(),
    }
}

fn scheme_and_addresses() -> impl Strategy<Value = (Scheme, Vec<Address>)> {
    (2u32..=64)
        .prop_flat_map(|base| {
            let s = scheme_for(base);
            let caps: Vec<u32> = (0..s.levels.len()).map(|i| s.capacity(i)).collect();
            let one = (1..=caps.len()).prop_flat_map(move |depth| {
                caps[..depth].iter().map(|&c| 0..c).collect::<Vec<_>>()
            });
            (Just(s), prop::collection::vec(one, 1..20))
        })
        .prop_map(|(s, values)| {
            let addresses = values
                .into_iter()
                .map(|v| Address::new(&s, v).unwrap())
                .collect();
            (s, addresses)
        })
}

/// Whether the disk holding `dir` treats `a` and `A` as the same name.
fn case_insensitive(dir: &std::path::Path) -> bool {
    let probe = dir.join("case-probe");
    fs::create_dir(&probe).unwrap();
    let insensitive = dir.join("CASE-PROBE").exists();
    fs::remove_dir(&probe).unwrap();
    insensitive
}

/// `PROPTEST_CASES` when set (the weekly CI run), otherwise `default`.
fn cases(default: u32) -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases(200)))]

    #[test]
    fn folders_round_trip((s, addresses) in scheme_and_addresses()) {
        s.validate().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let mut wanted: BTreeSet<Address> = addresses.into_iter().collect();
        // On a case-insensitive disk (macOS, Windows), numbers that differ
        // only in case are one folder: portability() warns about exactly
        // that, and case_collisions_are_real_where_reported checks the
        // warning. Here, keep one number per folder.
        if case_insensitive(dir.path()) {
            let mut seen = BTreeSet::new();
            wanted.retain(|a| seen.insert(a.render(&s).unwrap().to_lowercase()));
        }
        for a in &wanted {
            fs::create_dir(dir.path().join(name::render(&s, a, Some("Label")).unwrap())).unwrap();
        }
        // A `+` child, a dot entry and a plain folder sit beside the numbers.
        for other in ["+ Wendy", ".jd.ncl", "Notes"] {
            fs::create_dir(dir.path().join(other)).unwrap();
        }
        let mut found = BTreeSet::new();
        for entry in fs::read_dir(dir.path()).unwrap() {
            let file_name = entry.unwrap().file_name().into_string().unwrap();
            match name::parse(&s, &file_name) {
                Parsed::Numbered { address, label } => {
                    prop_assert_eq!(label.as_deref(), Some("Label"));
                    found.insert(address);
                }
                Parsed::Skip => prop_assert!(["+ Wendy", ".jd.ncl", "Notes"].contains(&file_name.as_str())),
            }
        }
        prop_assert_eq!(found, wanted);
    }

    // Byte order of the folder names is the tree order, where the scheme says
    // it is (ascending alphabets).
    #[test]
    fn folder_name_order((s, addresses) in scheme_and_addresses()) {
        prop_assume!(s.sorts_by_name());
        let level = addresses[0].level();
        let mut same: Vec<&Address> = addresses.iter().filter(|a| a.level() == level).collect();
        same.sort();
        same.dedup();
        let mut names: Vec<String> = same.iter().map(|a| a.render(&s).unwrap()).collect();
        let rendered = names.clone();
        names.sort();
        prop_assert_eq!(names, rendered);
    }
}

#[test]
fn base64url_reads_its_own_dash_as_a_digit() {
    let s = scheme_for(64);
    let area = Address::new(&s, vec![62]).unwrap();
    // Digit 62 is `-` and 63 is `_`.
    assert_eq!(area.render(&s).unwrap(), "-A~-_");
    assert_eq!(Address::parse(&s, "-A~-_"), Some(area));
    let id = Address::new(&s, vec![63, 62, 63 * 64 + 62]).unwrap();
    assert_eq!(id.render(&s).unwrap(), "_-._-");
    assert_eq!(Address::parse(&s, "_-._-"), Some(id));
}

#[test]
fn custom_alphabets_are_read_exactly() {
    let s = scheme_for(62);
    let id = Address::new(&s, vec![1, 36, 10]).unwrap();
    assert_eq!(id.render(&s).unwrap(), "1a.0A");
    assert_eq!(
        Address::parse(&s, "1A.0A"),
        Some(Address::new(&s, vec![1, 10, 10]).unwrap())
    );
    // Default digits stay case-insensitive.
    let hex = scheme_for(16);
    assert_eq!(Address::parse(&hex, "1f.0a"), Address::parse(&hex, "1F.0A"));
}

#[test]
fn bases_above_36_need_an_alphabet() {
    let mut s = scheme_for(40);
    s.alphabet = None;
    assert!(s.validate().is_err());
    let mut s = scheme_for(64);
    s.range_separator = "-".into();
    assert!(s.validate().is_err(), "`-` is a digit in base64url");
    let mut s = scheme_for(10);
    s.alphabet = Some("0123456780".into());
    assert!(s.validate().is_err(), "repeated digit");
    s.alphabet = Some("012345678/".into());
    assert!(s.validate().is_err(), "slash can't be in a file name");
}

#[test]
fn portability_warnings() {
    assert!(jd::scheme().portability().is_empty());
    assert!(scheme_for(36).portability().is_empty());
    // Base 62 has `a` and `A`: one folder on macOS and Windows disks.
    assert!(
        scheme_for(62)
            .portability()
            .contains(&Portability::CaseCollision('A', 'a'))
    );
    let mut s = scheme_for(16);
    s.range_separator = ":".into();
    assert_eq!(s.portability(), vec![Portability::WindowsForbidden(':')]);
    assert!(scheme_for(58).sorts_by_name());
    assert!(!scheme_for(64).sorts_by_name());
}

#[test]
fn base58_alphabet_works() {
    let s = Scheme {
        base: 58,
        alphabet: Some(alphabets::BASE58.into()),
        range_separator: "-".into(),
        levels: levels(),
    };
    s.validate().unwrap();
    let id = Address::new(&s, vec![0, 57, 58 * 58 - 1]).unwrap();
    assert_eq!(id.render(&s).unwrap(), "1z.zz");
    assert_eq!(Address::parse(&s, "1z.zz"), Some(id));
    assert_eq!(Address::parse(&s, "1O.zz"), None, "no O in base58");
}

/// `portability()` is a claim about real disks: where it reports a case
/// collision, two numbers really are one folder on a case-insensitive disk
/// (macOS and Windows by default; CI runs this on macOS), and where it
/// reports none, they never are.
#[test]
fn case_collisions_are_real_where_reported() {
    let dir = tempfile::tempdir().unwrap();
    let case_insensitive = case_insensitive(dir.path());

    // base62: `a` (36) and `A` (10) differ only in case.
    let s = scheme_for(62);
    assert!(
        s.portability()
            .contains(&Portability::CaseCollision('A', 'a'))
    );
    let upper = Address::new(&s, vec![1, 1, 10]).unwrap();
    let lower = Address::new(&s, vec![1, 1, 36]).unwrap();
    fs::create_dir(dir.path().join(upper.render(&s).unwrap())).unwrap();
    let second = fs::create_dir(dir.path().join(lower.render(&s).unwrap()));
    assert_eq!(second.is_err(), case_insensitive, "{second:?}");

    // base36 with the default digits reports nothing, and every number of a
    // level gets its own folder even on a case-insensitive disk.
    let s = scheme_for(36);
    assert!(s.portability().is_empty());
    let ids = tempfile::tempdir().unwrap();
    for id in 0..s.capacity(2) {
        let a = Address::new(&s, vec![1, 1, id]).unwrap();
        fs::create_dir(ids.path().join(a.render(&s).unwrap())).unwrap();
    }
    assert_eq!(
        fs::read_dir(ids.path()).unwrap().count(),
        s.capacity(2) as usize
    );
}
