//! The contracts in `nickel/xd.ncl` and `Scheme::validate` are two
//! definitions of a valid scheme. They must agree: a configuration loads
//! exactly when `validate` accepts its scheme, and loads unchanged. The
//! contracts may be looser than Rust (Rust then reports the error), never
//! stricter, or a scheme the core supports couldn't be written in Nickel.

use proptest::prelude::*;
use std::fmt::Write;
use std::fs;
use xd_conformance::{cases, strategies};
use xd_core::{Level, Scheme};
use xd_nickel::{LoadError, load};

/// A Nickel string literal. `%{` would start interpolation, so `%` is
/// escaped along with quotes and backslashes.
fn string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' | '\\' | '%' => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn to_nickel(s: &Scheme) -> String {
    let mut out = String::from("let xd = import \"xd.ncl\" in\n{ scheme = {\n");
    writeln!(out, "  base = {},", s.base).unwrap();
    if let Some(a) = &s.alphabet {
        writeln!(out, "  alphabet = {},", string(a)).unwrap();
    }
    writeln!(out, "  range_separator = {},", string(&s.range_separator)).unwrap();
    out.push_str("  levels = [\n");
    for l in &s.levels {
        writeln!(
            out,
            "    {{ name = {}, width = {}, separator = {}, range = {} }},",
            string(&l.name),
            l.width,
            string(&l.separator),
            l.range
        )
        .unwrap();
    }
    out.push_str("  ],\n} } | xd.System\n");
    out
}

fn load_scheme(s: &Scheme) -> Result<Scheme, LoadError> {
    let file = tempfile::Builder::new().suffix(".ncl").tempfile().unwrap();
    fs::write(file.path(), to_nickel(s)).unwrap();
    load(file.path()).map(|system| system.scheme)
}

/// Schemes near the edge of validity: generated ones with one thing broken.
fn edgy_scheme() -> impl Strategy<Value = Scheme> {
    (
        strategies::raw_scheme(),
        0u8..10,
        any::<prop::sample::Index>(),
    )
        .prop_map(|(mut s, edit, at)| {
            let i = at.index(s.levels.len());
            match edit {
                0 => s.base = at.index(4) as u32, // 0 to 3
                1 => s.base = 37 + at.index(30) as u32,
                2 => s.levels[i].width = 0,
                3 => s.levels[i].name.clear(),
                4 => s.range_separator.clear(),
                5 => s.range_separator = " ".into(),
                6 => s.levels.clear(),
                7 => s.alphabet = s.alphabet.take().map(|a| a[1..].to_string()),
                8 => s.levels[i].separator = "1".into(),
                _ => {}
            }
            s
        })
}

fn agree(s: &Scheme) -> Result<(), TestCaseError> {
    match (s.validate(), load_scheme(s)) {
        (Ok(()), Ok(loaded)) => prop_assert_eq!(&loaded, s),
        (Ok(()), Err(e)) => prop_assert!(
            false,
            "Rust accepts, Nickel rejects: {}\n{}",
            e,
            to_nickel(s)
        ),
        (Err(_), Ok(_)) => prop_assert!(false, "Rust rejects, but it loaded:\n{}", to_nickel(s)),
        (Err(_), Err(_)) => {}
    }
    Ok(())
}

proptest! {
    // Each case evaluates a Nickel program, so fewer cases than the core.
    #![proptest_config(ProptestConfig::with_cases(cases(128)))]

    #[test]
    fn valid_schemes_load_unchanged(s in strategies::scheme()) {
        agree(&s)?;
    }

    #[test]
    fn contracts_and_validate_agree_near_the_edge(s in edgy_scheme()) {
        agree(&s)?;
    }
}

#[test]
fn an_empty_level_name_is_rejected_by_both() {
    let mut s = xd_core::jd::scheme();
    s.levels[0] = Level {
        name: String::new(),
        ..s.levels[0].clone()
    };
    agree(&s).unwrap();
}
