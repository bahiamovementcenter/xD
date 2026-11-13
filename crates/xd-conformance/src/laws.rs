//! The laws, from `docs/jd-formal.md` 1.5 and `docs/design-patterns.md`
//! ("Lenses"). Each holds for every scheme `Scheme::validate` accepts.

use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use xd_core::name::{self, Parsed};
use xd_core::{Address, Scheme};

/// PutGet: what you write is what you read, `parse(render(a)) = a`.
pub fn put_get(s: &Scheme, a: &Address) -> Result<(), TestCaseError> {
    let rendered = a.render(s).map_err(fail)?;
    prop_assert_eq!(
        Address::parse(s, &rendered),
        Some(a.clone()),
        "{}",
        rendered
    );
    Ok(())
}

/// GetPut for folder names: a canonical name read and written back is the
/// same name, label included.
pub fn get_put_name(s: &Scheme, a: &Address, label: &str) -> Result<(), TestCaseError> {
    let written = name::render(s, a, Some(label)).map_err(fail)?;
    match name::parse(s, &written) {
        Parsed::Numbered { address, label: l } => {
            prop_assert_eq!(&address, a);
            let again = name::render(s, &address, l.as_deref()).map_err(fail)?;
            prop_assert_eq!(again, written);
        }
        Parsed::Skip => prop_assert!(false, "`{}` wasn't read as a number", written),
    }
    Ok(())
}

/// Every name `parse` accepts is canonical: rendering what it read gives the
/// same text back. With the default digits, letter case is the one allowed
/// difference (they are read in either case, written upper case).
pub fn parse_accepts_only_canonical(s: &Scheme, input: &str) -> Result<(), TestCaseError> {
    if let Some(a) = Address::parse(s, input) {
        let rendered = a.render(s).map_err(fail)?;
        if s.alphabet.is_some() {
            prop_assert_eq!(rendered, input);
        } else {
            prop_assert!(
                rendered.eq_ignore_ascii_case(input),
                "`{}` was read as {}, written `{}`",
                input,
                a,
                rendered
            );
        }
    }
    Ok(())
}

/// Siblings' integer form (experiments E2) and, where the scheme promises
/// it, their names in byte order, both follow the tree order.
pub fn sibling_order(s: &Scheme, a: &Address, b: &Address) -> Result<(), TestCaseError> {
    prop_assert_eq!(a.level(), b.level());
    let (ia, ib) = (
        a.to_integer(s).map_err(fail)?,
        b.to_integer(s).map_err(fail)?,
    );
    prop_assert_eq!(ia.cmp(&ib), a.cmp(b));
    if s.sorts_by_name() {
        let (ra, rb) = (a.render(s).map_err(fail)?, b.render(s).map_err(fail)?);
        prop_assert_eq!(ra.cmp(&rb), a.cmp(b), "`{}` vs `{}`", ra, rb);
    }
    Ok(())
}

/// Two different numbers never become the same folder on a disk that
/// ignores case, unless `Scheme::portability` says they can.
pub fn case_safe_unless_reported(
    s: &Scheme,
    a: &Address,
    b: &Address,
) -> Result<(), TestCaseError> {
    let collides = s
        .portability()
        .iter()
        .any(|p| matches!(p, xd_core::Portability::CaseCollision(..)));
    if a != b && !collides {
        let (ra, rb) = (a.render(s).map_err(fail)?, b.render(s).map_err(fail)?);
        prop_assert!(
            ra.to_lowercase() != rb.to_lowercase(),
            "`{}` and `{}` are the same folder on macOS, but no collision was reported",
            ra,
            rb
        );
    }
    Ok(())
}

fn fail(e: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(e.to_string())
}
