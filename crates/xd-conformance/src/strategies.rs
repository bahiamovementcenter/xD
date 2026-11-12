//! Generators for schemes, addresses and names.

use proptest::prelude::*;
use xd_core::{Address, Level, Scheme, SchemeError, alphabets};

/// Separators a level may use. Some clash with some alphabets (`_` is a
/// base64url digit); [`scheme`] drops those schemes.
const SEPARATORS: &[&str] = &["", "", ".", "_", "+", ","];

/// The digits for `base`: the default `0-9A-Z` up to 36, otherwise a prefix
/// of base62 or base64url. `pick` chooses between the options a base has,
/// so small bases are also tried with case-sensitive custom alphabets.
fn alphabet(base: u32, pick: u8) -> Option<String> {
    let prefix = |a: &str| a[..base as usize].to_string();
    match (base, pick % 3) {
        (..=36, 0 | 1) => None,
        (..=58, 2) => Some(prefix(alphabets::BASE58)),
        (..=62, _) => Some(prefix(alphabets::BASE62)),
        _ => Some(prefix(alphabets::BASE64URL)),
    }
}

fn level() -> impl Strategy<Value = Level> {
    (
        "[a-z]{1,8}",
        1u32..=4,
        prop::sample::select(SEPARATORS),
        any::<bool>(),
    )
        .prop_map(|(name, width, separator, range)| Level {
            name,
            width,
            separator: separator.to_string(),
            range,
        })
}

/// Any scheme, valid or not: bases 2 to 64, one to five levels.
pub fn raw_scheme() -> impl Strategy<Value = Scheme> {
    (
        2u32..=64,
        any::<u8>(),
        prop::sample::select(&["-", "~"][..]),
        prop::collection::vec(level(), 1..=5),
    )
        .prop_map(|(base, pick, range_separator, levels)| Scheme {
            base,
            alphabet: alphabet(base, pick),
            range_separator: range_separator.to_string(),
            levels,
        })
}

/// A scheme [`Scheme::validate`] accepts. Generated schemes are repaired
/// toward validity (levels dropped until an address fits in 64 bits, no
/// range on the last level, a range separator that isn't a digit) and
/// dropped if still invalid, so the generator stays wide without rejecting
/// most of what it makes.
pub fn scheme() -> impl Strategy<Value = Scheme> {
    raw_scheme().prop_filter_map("scheme still invalid after repair", |mut s| {
        if s.alphabet.as_deref().is_some_and(|a| a.contains('-')) {
            s.range_separator = "~".into();
        }
        while matches!(
            s.validate(),
            Err(SchemeError::TooLarge | SchemeError::TooWide(_))
        ) && s.levels.len() > 1
        {
            s.levels.pop();
        }
        if let Some(last) = s.levels.last_mut() {
            last.range = false;
        }
        s.validate().is_ok().then_some(s)
    })
}

/// An address that fits `s`, at any depth.
pub fn address(s: &Scheme) -> impl Strategy<Value = Address> + use<> {
    let caps: Vec<u32> = (0..s.levels.len()).map(|i| s.capacity(i)).collect();
    let s = s.clone();
    (1..=caps.len())
        .prop_flat_map(move |depth| caps[..depth].iter().map(|&c| 0..c).collect::<Vec<_>>())
        .prop_map(move |values| Address::new(&s, values).expect("values fit the capacities"))
}

/// An address at exactly `level`.
pub fn address_at(s: &Scheme, level: usize) -> impl Strategy<Value = Address> + use<> {
    let s = s.clone();
    (0..=level)
        .map(|i| 0..s.capacity(i))
        .collect::<Vec<_>>()
        .prop_map(move |values| Address::new(&s, values).expect("values fit the capacities"))
}

pub fn scheme_and_address() -> impl Strategy<Value = (Scheme, Address)> {
    scheme().prop_flat_map(|s| (Just(s.clone()), address(&s)))
}

/// A scheme and two addresses on the same level of it.
pub fn scheme_and_siblings() -> impl Strategy<Value = (Scheme, Address, Address)> {
    scheme().prop_flat_map(|s| {
        (0..s.levels.len()).prop_flat_map(move |level| {
            (
                Just(s.clone()),
                address_at(&s, level),
                address_at(&s, level),
            )
        })
    })
}

/// A label as people write them: doesn't start with a space (the first
/// space ends the number), may contain more.
pub fn label() -> impl Strategy<Value = String> {
    "[A-Za-z0-9äé&()'][A-Za-z0-9äé&()' ._+-]{0,20}"
}

/// Strings that look like numbers of `s`: its digits, separators and range
/// separator, mixed with some noise, so a parser's interesting branches are
/// reached far more often than by arbitrary text.
pub fn number_like(s: &Scheme) -> impl Strategy<Value = String> + use<> {
    let mut pieces: Vec<String> = match &s.alphabet {
        Some(a) => a.chars().map(String::from).collect(),
        None => "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .chars()
            .map(String::from)
            .collect(),
    };
    pieces.extend(s.levels.iter().map(|l| l.separator.clone()));
    pieces.push(s.range_separator.clone());
    pieces.extend([" ", "é", "/", ""].map(String::from));
    prop::collection::vec(prop::sample::select(pieces), 0..12).prop_map(|p| p.concat())
}

/// A rendered number of `s` with one edit: a character replaced, inserted
/// or removed, using `s`'s own digits and separators. Most results are
/// almost valid, which is where a parser's off-by-one mistakes are.
pub fn near_number(s: &Scheme) -> impl Strategy<Value = String> + use<> {
    // The digits, plus the first one past the base in both cases: the
    // likeliest off-by-one.
    let mut pieces: Vec<char> = match &s.alphabet {
        Some(a) => a.chars().collect(),
        None => (0..=s.base.min(35))
            .filter_map(|v| char::from_digit(v, 36))
            .flat_map(|c| [c, c.to_ascii_uppercase()])
            .collect(),
    };
    pieces.extend(s.levels.iter().flat_map(|l| l.separator.chars()));
    pieces.extend(s.range_separator.chars());
    pieces.push(' ');
    let s2 = s.clone();
    (
        address(s),
        any::<prop::sample::Index>(),
        prop::sample::select(pieces),
        0u8..4,
    )
        .prop_map(move |(a, at, c, edit)| {
            let mut chars: Vec<char> = a.render(&s2).expect("fits").chars().collect();
            let i = at.index(chars.len() + 1);
            match edit {
                0 if i < chars.len() => chars[i] = c,
                1 => chars.insert(i, c),
                2 if i < chars.len() => {
                    chars.remove(i);
                }
                _ => {}
            }
            chars.into_iter().collect()
        })
}
