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

## Expanding the library

When a configuration fails with `capability '<code> isn't built yet`, the
library lacks something the system needs. Don't work around it in the
configuration. Follow `.claude/skills/expand-library/SKILL.md` (ADR 0007):

1. Look `<code>` up in `nickel/capabilities.ncl`.
2. Write the system you're working on as a config in `nickel/examples/`,
   and add it to the capability's `configs`. It fails with that code.
3. If the capability now has one config, stop: it's recorded, and waits for
   a second consumer. Tell the user it's deferred and why.
4. If it already had one, or a system in its `known` list fits, write that
   one as a second config too. Two configs mean it's time to build it, and
   `cargo test` says so until it's built.
5. Build it: an ADR first (or extend the one named in `adr`), then the
   cheapest mechanism in `docs/extensibility.md` that serves both configs,
   then mark it `'built` and make both configs pass. One capability per pull
   request.

A system only counts once it is a config: a name in `known` is a candidate,
not a consumer. Anything that would change a sealed rule in
`docs/extensibility.md` stops at the ADR for the owner's approval, and an
agent never merges.

## Build and test

From `nix develop`:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
nickel format nickel/*.ncl nickel/examples/*.ncl && nickel typecheck nickel/jd.ncl   # when Nickel files change
```

CI (`.github/workflows/ci.yml`) runs the same in the Nix dev shell. Testing is described
in `docs/testing.md`: a promise of the formal model is a law in `crates/xd-conformance`,
checked by property tests; a bug becomes a named test in `crates/xd-core/tests/regressions.rs`
before it's fixed. `docs/reference/` is vendored and stays out of tests and formatting.

## Commits

Short imperative subject starting with the area (`core: …`, `docs: …`), one logical
change per commit. Work on a branch and open a pull request against `main`.
