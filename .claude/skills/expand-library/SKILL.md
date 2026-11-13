---
name: expand-library
description: Use when an xD configuration fails with "capability '<code> isn't built yet", or a system design needs something xD's Nickel library can't express. Records the need, and builds the capability once two configs need it.
---

# Expanding xD's library

xD builds a mechanism only when two real systems need it
(`docs/extensibility.md`). "Real" means written as a config: a Nickel file
under `nickel/` that uses the capability. `nickel/capabilities.ncl` is the
registry, and `crates/xd-nickel/tests/capabilities.rs` enforces it (ADR 0007).

## 1. Identify the capability

The error names it: `capability 'variable_width isn't built yet: …`. If you
hit a gap with no capability error (the library accepts something and does
the wrong thing, or there's no way to write what you need), find the closest
code in `capabilities.ncl`. If none fits, add one with `status = 'deferred`,
a one-line `summary`, and the systems from the survey that would need it in
`known`, and make `xd.ncl` report it where a config would ask for it (use
`unbuilt`, `Unbuilt` or `unbuilt_message`).

## 2. Record your system as a consumer

Write the system you're working on as `nickel/examples/<system>.ncl`:

- import `xd.ncl` (and units such as `jd.ncl`) as the other examples do;
- build its pattern or configuration as you'd want to write it once the
  capability exists;
- end with `std.test.assert_all` over a few round trips (`render`, `parse`,
  `write`), so that once it's built, the example proves it works;
- a header comment naming the system, the capability, and the source for
  any fact about the system.

Add its path to the capability's `configs`. Run
`cargo test -p xd-nickel --test capabilities`: the example must fail with
the capability's code.

## 3. One config: stop

If it's the first config, the capability is deferred. Commit the example and
the registry change, and tell the user: what's missing, that it's recorded,
and which `known` systems would make it the second.

## 4. Two configs: it's time

If the capability already had a config, or a system in `known` is close to
yours, write that system as a second example too, from its source (the
survey in the project files cites one for each). Two configs make the
capabilities test fail with "it's time to build it". Then:

1. **ADR.** If `adr` names one, extend it; otherwise write the next numbered
   ADR in `docs/adr/`. Design for both configs, not just yours.
2. **Cheapest mechanism.** Follow `docs/extensibility.md`: a Nickel value or
   contract before a core change, a closed enum before a trait.
3. **Build it** in `nickel/xd.ncl` (and the core, if the core must hold it;
   the `core_*` capabilities track that separately).
4. **Mark it built**: `status = 'built`, `adr` set. Both configs must now
   evaluate, with their assertions passing.
5. **Laws.** If it changes what `render` or `parse` do, add or extend the
   property tests in `crates/xd-nickel/tests/primitives.rs` so the laws of
   `docs/testing.md` cover it.
6. **Check**: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`, `nickel format --check nickel/*.ncl nickel/examples/*.ncl`.

One capability per pull request.

## Limits

- Anything that changes a sealed rule in `docs/extensibility.md` stops at
  the ADR: open the pull request with the ADR only, and ask the owner.
- Never merge. Never lower the two-config rule, mark a capability built
  without both configs passing, or mark one `grandfathered`.
- If `nickel/xd.ncl`'s file layout or field names change, tell the user:
  rust-nickel vendors a copy.
