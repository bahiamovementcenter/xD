# AGENTS.md

xD (Extensible Decimal): composable primitives for organizational systems. Johnny.Decimal
is one configuration and the default; the core must not assume it.

- Design decisions are ADRs in `docs/adr/`, numbered in order. A change to the design
  gets an ADR with it. The formal model is `docs/jd-formal.md`; ideas that aren't JD are
  in `docs/jd-experiments.md`.
- Defaults follow the Johnny.Decimal spec (johnnydecimal.com), not the `jd` CLI.
- Keep the core pure: no file system access outside the apply step and tests' temporary
  directories.

## Build and test

From `nix develop`:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Commits

Short imperative subject starting with the area (`core: …`, `docs: …`), one logical
change per commit. Work on a branch and open a pull request against `main`.
