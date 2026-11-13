# AGENTS.md

xD (Extensible Decimal): composable primitives for organizational systems. Johnny.Decimal
is one configuration and the default; the core must not assume it.

- Design decisions are ADRs in `docs/adr/`, numbered in order. A change to the design
  gets an ADR with it. The formal model is `docs/jd-formal.md`; ideas that aren't JD are
  in `docs/jd-experiments.md`.
- Defaults follow the Johnny.Decimal spec (johnnydecimal.com), not the `jd` CLI.
- A new extension point follows `docs/extensibility.md`: the cheapest mechanism that
  works, no trait or plugin interface before a second implementation, and the sealed
  rules there stay sealed unless an ADR changes them.
- Keep the core pure: no file system access outside the apply step and tests' temporary
  directories.

## Build and test

From `nix develop`:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
nickel format nickel/*.ncl && nickel typecheck nickel/jd.ncl   # when Nickel files change
```

CI (`.github/workflows/ci.yml`) runs the same in the Nix dev shell. Testing is described
in `docs/testing.md`: a promise of the formal model is a law in `crates/xd-conformance`,
checked by property tests; a bug becomes a named test in `crates/xd-core/tests/regressions.rs`
before it's fixed. `docs/reference/` is vendored and stays out of tests and formatting.

## Commits

Short imperative subject starting with the area (`core: …`, `docs: …`), one logical
change per commit. Work on a branch and open a pull request against `main`.
