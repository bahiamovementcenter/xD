use std::fs;
use xd_core::{Address, jd};
use xd_nickel::{LoadError, library_dir, load};

#[test]
fn jd_ncl_is_the_jd_scheme() {
    let system = load(library_dir().join("jd.ncl")).unwrap();
    assert_eq!(system.scheme, jd::scheme());
}

fn write(src: &str) -> tempfile::TempPath {
    let file = tempfile::Builder::new().suffix(".ncl").tempfile().unwrap();
    fs::write(file.path(), src).unwrap();
    file.into_temp_path()
}

#[test]
fn a_system_can_build_on_jd() {
    // Everything is configurable: here, JD with hex digits.
    let path = write(r#"(import "jd.ncl") & { scheme.base | force = 16 }"#);
    let system = load(&path).unwrap();
    let s = system.scheme;
    assert_eq!(s.base, 16);
    assert_eq!(Address::new(&s, vec![1]).unwrap().render(&s), "10-1F");
}

#[test]
fn custom_alphabets_load() {
    let path = write(
        r#"
let xd = import "xd.ncl" in
{
  scheme = {
    base = 58,
    alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz",
    levels = [{ name = "box", width = 1, range = true }, { name = "item", width = 2, separator = "." }],
  },
} | xd.System
"#,
    );
    let s = load(&path).unwrap().scheme;
    assert_eq!(Address::new(&s, vec![57, 0]).unwrap().render(&s), "z.11");
}

#[test]
fn contracts_reject_bad_configurations() {
    let path = write(r#"(import "jd.ncl") & { scheme.base | force = 1 }"#);
    assert!(matches!(load(&path), Err(LoadError::Nickel(_))));
    let path = write(r#"(import "jd.ncl") & { scheme.levels | force = [] }"#);
    assert!(matches!(load(&path), Err(LoadError::Nickel(_))));
}

#[test]
fn rust_validation_catches_what_contracts_dont() {
    // Base 40 passes the contract but has no alphabet.
    let path = write(r#"(import "jd.ncl") & { scheme.base | force = 40 }"#);
    assert!(matches!(load(&path), Err(LoadError::Scheme(_))));
}
