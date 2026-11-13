//! xD's primitives in `nickel/xd.ncl` (ADR 0004) are a second
//! implementation of the core's numbers: `xd.pattern` renders and parses in
//! Nickel, for programs that embed Nickel without xD's core. Every pattern
//! the core can hold must lower to the same scheme and agree with the core
//! on every name, so the laws stay one law with two implementations.

use nickel_embed::{Program, Source};
use proptest::prelude::*;
use serde_json::{Value, json};
use std::fmt::Write;
use xd_conformance::{cases, strategies};
use xd_core::{Address, Scheme, name};
use xd_nickel::library_dir;

/// A Nickel string literal (see agreement.rs).
fn string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        if matches!(c, '"' | '\\' | '%') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

fn eval(src: &str) -> Result<Value, nickel_embed::Error> {
    let mut program = Program::builder()
        .source(Source::code(src))
        .import_path(library_dir())
        .build()?;
    let json = program.export(nickel_embed::ExportFormat::Json)?;
    Ok(serde_json::from_str(&json).expect("Nickel exports valid JSON"))
}

/// The scheme as a pattern of primitives: each level's separator becomes a
/// literal before its slot, and every name ends in a space and a label.
fn to_pattern(s: &Scheme) -> String {
    let alphabet = match &s.alphabet {
        None => format!("xd.alphabets.first {}", s.base),
        Some(a) => format!("xd.alphabets.of {}", string(a)),
    };
    let mut parts = String::new();
    for l in &s.levels {
        if !l.separator.is_empty() {
            writeln!(parts, "  xd.literal {},", string(&l.separator)).unwrap();
        }
        writeln!(
            parts,
            "  xd.slot {{ name = {}, width = {}, alphabet = a, range = {}, range_separator = {} }},",
            string(&l.name),
            l.width,
            l.range,
            string(&s.range_separator)
        )
        .unwrap();
    }
    format!(
        "let xd = import \"xd.ncl\" in\nlet a = {alphabet} in\nxd.pattern [\n{parts}  xd.sep.label,\n  xd.label,\n]"
    )
}

/// What the core makes of a name, in the shape `pattern.parse` returns.
fn core_parse(s: &Scheme, input: &str) -> Value {
    match name::parse(s, input) {
        name::Parsed::Numbered { address, label } => {
            let mut v = json!({ "address": address.values() });
            if let Some(label) = label {
                v["label"] = json!(label);
            }
            v
        }
        name::Parsed::Skip => Value::Null,
    }
}

fn agree(
    s: &Scheme,
    addresses: &[Address],
    labels: &[String],
    inputs: &[String],
) -> Result<(), TestCaseError> {
    let values: Vec<String> = addresses
        .iter()
        .map(|a| serde_json::to_string(a.values()).unwrap())
        .collect();
    let names: Vec<String> = addresses
        .iter()
        .zip(labels.iter().cycle())
        .map(|(a, l)| name::render(s, a, Some(l)).unwrap())
        .collect();
    let to_parse: Vec<&String> = names.iter().chain(inputs).collect();
    let src = format!(
        "let p = {} in {{\n  scheme = p.scheme,\n  rendered = [{}],\n  named = [{}],\n  parsed = [{}],\n}}",
        to_pattern(s),
        values
            .iter()
            .map(|v| format!("p.render {v}"))
            .collect::<Vec<_>>()
            .join(", "),
        values
            .iter()
            .zip(labels.iter().cycle())
            .map(|(v, l)| format!("p.name {v} {}", string(l)))
            .collect::<Vec<_>>()
            .join(", "),
        to_parse
            .iter()
            .map(|i| format!("p.parse {}", string(i)))
            .collect::<Vec<_>>()
            .join(", "),
    );
    let got = eval(&src).map_err(|e| TestCaseError::fail(format!("{e}\n{src}")))?;

    let mut scheme: Scheme = serde_json::from_value(got["scheme"].clone()).unwrap();
    // A range separator only shows in a scheme with a range level; the
    // pattern keeps it on the slots, so without one it falls back to "-".
    if !s.levels.iter().any(|l| l.range) {
        scheme.range_separator.clone_from(&s.range_separator);
    }
    prop_assert_eq!(&scheme, s, "the pattern lowers to another scheme:\n{}", src);
    let rendered: Vec<String> = addresses.iter().map(|a| a.render(s).unwrap()).collect();
    prop_assert_eq!(&got["rendered"], &json!(rendered), "{}", src);
    prop_assert_eq!(&got["named"], &json!(names), "{}", src);
    let parsed: Vec<Value> = to_parse.iter().map(|i| core_parse(s, i)).collect();
    prop_assert_eq!(&got["parsed"], &json!(parsed), "{}", src);
    Ok(())
}

fn case() -> impl Strategy<Value = (Scheme, Vec<Address>, Vec<String>, Vec<String>)> {
    strategies::scheme().prop_flat_map(|s| {
        (
            Just(s.clone()),
            prop::collection::vec(strategies::address(&s), 1..6),
            prop::collection::vec(strategies::label(), 1..3),
            prop::collection::vec(
                prop_oneof![strategies::number_like(&s), strategies::near_number(&s)],
                0..8,
            ),
        )
    })
}

proptest! {
    // Each case evaluates a Nickel program, so fewer cases than the core.
    #![proptest_config(ProptestConfig::with_cases(cases(64)))]

    #[test]
    fn patterns_agree_with_the_core((s, addresses, labels, inputs) in case()) {
        agree(&s, &addresses, &labels, &inputs)?;
    }
}

#[test]
fn jd_agrees_with_the_core() {
    let s = xd_core::jd::scheme();
    let addresses: Vec<Address> = [vec![0], vec![1], vec![1, 0], vec![1, 1, 11], vec![9, 9, 99]]
        .into_iter()
        .map(|v| Address::new(&s, v).unwrap())
        .collect();
    let inputs = ["10-19", "1", "10-18", "11.1", "11.11 ", "+ Wendy", "W0011"].map(String::from);
    agree(&s, &addresses, &["Passport".into()], &inputs).unwrap();
}

#[test]
fn the_nickel_examples_pass() {
    let src = std::fs::read_to_string(library_dir().join("tests/primitives.ncl")).unwrap();
    assert_eq!(eval(&src).unwrap(), json!(true));
}

#[test]
fn patterns_the_core_cant_hold_fail_to_lower() {
    let mixed = r#"let xd = import "xd.ncl" in
(xd.pattern [
  xd.slot { name = "category", width = 2 },
  xd.slot { name = "board", width = 2, alphabet = xd.alphabets.hex },
]).scheme"#;
    let e = eval(mixed).unwrap_err();
    assert!(e.to_string().contains("one alphabet"), "{e}");
}

#[test]
fn malformed_patterns_are_rejected() {
    for (parts, says) in [
        ("[]", "at least one slot"),
        (
            "[xd.label, xd.slot { name = \"id\", width = 2 }]",
            "last part",
        ),
        (
            "[xd.slot { name = \"id\", width = 2 }, xd.label]",
            "separator before",
        ),
        (
            "[xd.slot { name = \"area\", width = 1, range = true }]",
            "range",
        ),
        ("[xd.slot { name = \"id\", width = 0 }]", "PosNat"),
        ("[xd.slot { name = \"id\", width = 2, widht = 3 }]", "widht"),
    ] {
        let src = format!("let xd = import \"xd.ncl\" in (xd.pattern {parts}).render [0]");
        let e = eval(&src).unwrap_err().to_string();
        assert!(e.contains(says), "{parts}: expected `{says}` in\n{e}");
    }
}
