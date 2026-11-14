//! The capability registry, `nickel/capabilities.ncl` (ADR 0007), kept
//! honest. A capability counts a system as a consumer only once it is written
//! as a config under `nickel/`; two configs make an unbuilt capability ready
//! to build, and this test says so until it is.

use nickel_embed::{ExportFormat, Program, Source};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use xd_nickel::library_dir;

#[derive(Deserialize)]
struct Registry {
    capabilities: BTreeMap<String, Capability>,
}

#[derive(Deserialize)]
struct Capability {
    status: Status,
    grandfathered: bool,
    configs: Vec<String>,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
enum Status {
    Built,
    Proposed,
    Deferred,
}

fn registry() -> Registry {
    let mut program = Program::builder()
        .source(Source::file(library_dir().join("capabilities.ncl")))
        .build()
        .unwrap();
    let json = program.export(ExportFormat::Json).unwrap();
    serde_json::from_str(&json).expect("capabilities.ncl has the registry's shape")
}

/// Evaluates a config the way `nickel export` would, with nickel/ on the
/// import path; the error is rendered as plain text.
fn evaluate(config: &str) -> Result<(), String> {
    let path = library_dir().join(config);
    assert!(
        path.is_file(),
        "{config} is listed in capabilities.ncl but doesn't exist"
    );
    Program::builder()
        .source(Source::file(&path))
        .import_path(library_dir())
        .build()
        .and_then(|mut p| p.export(ExportFormat::Json))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn marker(code: &str) -> String {
    format!("capability '{code} isn't built yet")
}

#[test]
fn built_capabilities_have_two_working_configs() {
    for (code, c) in registry().capabilities {
        if c.status != Status::Built {
            continue;
        }
        if !c.grandfathered {
            assert!(
                c.configs.len() >= 2,
                "capability '{code} is built but has {} config(s): it needs two (ADR 0007)",
                c.configs.len()
            );
        }
        for config in &c.configs {
            if let Err(e) = evaluate(config) {
                panic!("{config}, a config of the built capability '{code}, fails:\n{e}");
            }
        }
    }
}

#[test]
fn unbuilt_capabilities_fail_with_their_code_until_a_second_consumer() {
    for (code, c) in registry().capabilities {
        if c.status == Status::Built {
            continue;
        }
        assert!(
            c.configs.len() < 2,
            "capability '{code} has two consumers ({}): it's time to build it. \
             Follow AGENTS.md, \"Expanding the library\".",
            c.configs.join(", ")
        );
        for config in &c.configs {
            match evaluate(config) {
                Ok(()) => panic!(
                    "{config} evaluates, but '{code} is {:?}: mark it built, or the config doesn't use it",
                    c.status
                ),
                Err(e) => assert!(
                    e.contains(&marker(&code)),
                    "{config} should fail with \"{}\", but fails with:\n{e}",
                    marker(&code)
                ),
            }
        }
    }
}

#[test]
fn every_code_xd_reports_is_registered() {
    let source = fs::read_to_string(library_dir().join("xd.ncl")).unwrap();
    let registered: BTreeSet<String> = registry().capabilities.into_keys().collect();
    // Codes appear in xd.ncl as the first argument of `unbuilt`, `Unbuilt` or
    // `unbuilt_message`, or as a `code = "..."` field, always as a literal.
    let mut used = BTreeSet::new();
    for prefix in [
        "unbuilt \"",
        "Unbuilt \"",
        "unbuilt_message \"",
        "code = \"",
        "=> \"",
    ] {
        for (i, _) in source.match_indices(prefix) {
            let rest = &source[i + prefix.len()..];
            let code = &rest[..rest.find('"').unwrap()];
            if code.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                used.insert(code.to_string());
            }
        }
    }
    assert!(!used.is_empty(), "found no capability codes in xd.ncl");
    let missing: Vec<_> = used.difference(&registered).collect();
    assert!(
        missing.is_empty(),
        "xd.ncl reports capabilities that capabilities.ncl doesn't list: {missing:?}"
    );
}

#[test]
fn every_example_is_some_capabilitys_config() {
    let listed: BTreeSet<String> = registry()
        .capabilities
        .into_values()
        .flat_map(|c| c.configs)
        .collect();
    let examples = library_dir().join("examples");
    for entry in fs::read_dir(&examples).unwrap() {
        let path = entry.unwrap().path();
        let relative = Path::new("examples").join(path.file_name().unwrap());
        let relative = relative.to_str().unwrap();
        assert!(
            listed.contains(relative),
            "{relative} isn't a config of any capability in capabilities.ncl"
        );
    }
}
