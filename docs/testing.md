# Testing strategy

How xD is tested: what each layer must prove, which kind of test proves it,
what exists today and what's missing. Written 2026-11-12 against the code on
this branch (`xd-core`, `xd-nickel`) and the design in ADR 0001 and ADR 0002.
Nothing here changes the design; where it proposes a rule for contributors,
that rule goes into `AGENTS.md` when it's adopted.

## Principles

1. **Laws before examples.** xD's promises are mostly laws: a number is never
   reused, a name read back is the number written, a plan undone is the tree
   before. A law is a property test; an example test documents one case of
   it. Write the law first, then the examples that explain it.
2. **The pure core carries the weight.** ADR 0001 keeps the core free of I/O,
   so almost everything (schemes, the registry, planning, checks) is tested
   in memory and in milliseconds. Tests that touch disks, jj or plugins are
   few and prove only the seam.
3. **One law, every implementation.** Each law is written once, as a
   function over a scheme or a projection, and run against every built-in
   implementation. That set becomes the conformance suite plugins must pass
   (extensibility.md, "Reference tests").
4. **Oracles where they exist.** The JD spec's examples and jdlint's findings
   are external truth. Tests compare against them instead of against what
   xD happened to do last time.
5. **A bug becomes a test first.** A failing case found by proptest or by
   hand is pinned as a named example before it's fixed (proptest's
   regression files are committed too).

## The pyramid for xD

```
              ┌──────────────────────────┐
              │  CLI end to end (few)    │  xd plan/apply/undo on a temp root,
              │                          │  under jj, golden output
            ┌─┴──────────────────────────┴─┐
            │  Seams (some)                │  Nickel loading, real folders,
            │                              │  apply backends, jj, WIT plugins,
            │                              │  jdlint oracle
          ┌─┴──────────────────────────────┴─┐
          │  Core laws and examples (most)   │  scheme, address, name, registry,
          │                                  │  plan, check: proptest + examples
          └──────────────────────────────────┘
```

Rough proportion as the code grows: about 70% core, 25% seams, 5% end to
end. `cargo test` must stay under a minute on a laptop; anything slower
moves to a scheduled job.

## The harness

| Where | What it holds |
| --- | --- |
| `crates/xd-core/tests/` | examples for the core (`core.rs`), real folders for bases 2–64 (`folders.rs`), and every bug found so far, pinned by name (`regressions.rs`) |
| `crates/xd-conformance/` | the conformance suite: generators (`strategies`) and laws (`laws`) anyone can call from their own `proptest!`, plus the suite run on generated schemes (`tests/laws.rs`) and the registry against a model (`tests/registry.rs`) |
| `crates/xd-nickel/tests/` | loading, and the agreement test between the Nickel contracts and `Scheme::validate` (`agreement.rs`) |
| `*.proptest-regressions` | seeds of real failures proptest found, committed so they run first |
| `.github/workflows/ci.yml` | every check in `AGENTS.md` on Linux, `cargo test` on macOS, and a weekly run with `PROPTEST_CASES=10000` |

Generated schemes cover bases 2 to 64, the default digits and base58,
base62 and base64url alphabets (so small bases are tried case-sensitive
too), one to five levels of width 1 to 4, five separators and both range
separators. Anything `validate` rejects is repaired or dropped, so every law
runs only on valid schemes.

## What's covered

| Promise | Test | Kind |
| --- | --- | --- |
| **PutGet**: `parse(render(a)) = a` | `laws::put_get` on generated schemes; every JD address exhaustively | property, exhaustive |
| **GetPut** for folder names: a canonical name read and written back is unchanged, label included | `laws::get_put_name` | property |
| `parse` accepts only canonical numbers (default digits: up to letter case) | `laws::parse_accepts_only_canonical` on number-like text and on rendered numbers with one edit | property |
| Siblings' integers and names sort in tree order | `laws::sibling_order` | property |
| Different numbers never share a folder on a case-insensitive disk unless `portability()` says so | `laws::case_safe_unless_reported`, and `case_collisions_are_real_where_reported` on the real disk (macOS in CI) | property, disk |
| `parse` never panics; nothing panics on a validated scheme | `parse_never_panics`, `validated_schemes_are_safe` | property |
| JD ranges parse only when exact (`10-19`, not `10-18`, `11-19`, `1`) | `jd_ranges_parse_only_when_exact` | examples |
| Real folders round trip | `folders.rs`, bases 2–64 | property on temp dirs |
| **Registry** (formal 1.3, 1.4): agrees with a naive model after every step of random insert/rename/retire/allocate sequences, with and without reserved numbers; active and retired never overlap; retired only grows; no orphans; allocate never hands out a used or reserved number, and what it hands out can be inserted | `registry_matches_the_model` | model-based property |
| Allocating until full hands out every non-reserved number exactly once | `allocation_fills_a_level_exactly` | property |
| JD allocates the first area as `10-19` | `jd_areas_are_allocated_after_the_system_area` | example |
| Nickel contracts and `validate` agree: a scheme loads exactly when `validate` accepts it, and loads unchanged | `agreement.rs`, on valid schemes and on schemes with one thing broken | differential property |
| `jd.ncl` is `xd_core::jd::scheme()`; open `System`, closed `Scheme` | `load.rs` | examples |

The tests catch real mistakes: an off-by-one in reading digits
(`v <= base`) fails `near_misses_are_rejected_or_canonical` on every run,
and dropping the never-reuse check or the retired record fails
`registry_matches_the_model`.

## Found and fixed

| Bug | Found by | Fix |
| --- | --- | --- |
| `Address::to_integer` overflowed for valid schemes whose levels add up to more than 64 bits (base 64, three 5-digit levels) | reading the code for this plan | `validate` rejects such schemes (`SchemeError::TooLarge`) |
| `Address::render` panicked for an address from another scheme | the same | `render`, `name::render` and `to_integer` return `Result`; `Address::check`; `allocate` rejects such a parent as `Invalid` |
| A scheme with an unnamed level was valid in Rust but rejected by the Nickel contract | the agreement test | `validate` requires a name (`SchemeError::LevelName`) |

## Still open

- **Non-canonical names aren't reported yet.** With the default digits,
  `1a.1f` (hex) is read as `1A.1F`. The law allows that; the plan should
  show the rename as a normalization once plans exist (design-patterns.md,
  "Lenses"), which needs `parse` to say whether a name was canonical.
- **Diagnostics.** Nickel's error for a misspelled field should name it; a
  snapshot test of the message (`insta`) when errors get their final form.
- **Windows.** CI runs Linux and macOS. Windows forbids characters
  `portability()` reports; a Windows job when someone uses xD there.
- **Fuzzing** with `cargo fuzz` on the parsers, once there's more parsing
  (the shorthand grammar, Org JDex). The `parse_never_panics` property
  covers today's.

## Tests for what comes next

These follow the order of work in ADR 0001 and ADR 0002. Each row is the
test to write with the feature, not after.

### Plan (ADR 0002)

| What | Kind | Example cases |
| --- | --- | --- |
| Three-way diff | table test, one case per row of ADR 0002's table, per entry kind | config renamed, disk same → `Rename`; disk renamed, config same → `RecordInConfig`; both renamed the same → nothing; both renamed differently → conflict, empty plan; first run without `xd-applied` → only config edits |
| Diff is level-triggered | property | `plan(c, cur, last)` depends only on its inputs: same inputs, same plan; order of folders on disk doesn't matter |
| Every operation inverts | property, with the `simulate` interpreter | for any tree and any checked plan: `simulate(invert(p), simulate(p, t)) = t` |
| Apply then plan is empty | property | `plan(c, simulate(plan(c, t, l), t), c) = []` (convergence) |
| Renames never delete | property | no plan contains a remove and a create for the same number |
| `check` rejects | examples, one per invariant | reused number, missing parent, two entries per number, a path outside the root |
| Skipping `check` doesn't compile | compile-fail test (`trybuild`) | `apply(Plan<Draft>)` |
| Hooks (`Plan -> Plan` in Nickel) | integration | a hook that adds an operation is checked like any other; a hook that reuses a number is rejected; a hook that errors stops the run |

### Apply and undo

Apply is the first code allowed to touch the file system, so its tests use
temporary roots only (as `AGENTS.md` requires).

- **Golden runs**: apply a plan to a temp root built from a fixture, compare
  the resulting tree (names only, sorted) with a committed snapshot
  (`insta`).
- **Rollback**: inject a failure at step *k* of *n* (a test backend that
  fails on request, or a read-only folder) and assert the tree equals the
  starting tree and the journal says so. Property over *k*.
- **Undo**: apply, undo, compare with the start; apply, edit by hand, undo
  → undo refuses or reports, never overwrites the hand edit.
- **Content is never deleted**: a file inside any folder survives every
  plan; property over random plans with a file in each folder.
- **jj**: one integration test that apply commits and moves
  `xd-applied/<host>`, and `xd undo` moves it back. Uses `jj-lib` in a temp
  repo; marked so it can be skipped where jj isn't available, but runs in CI.

### `xd check` and the jdlint oracle

ADR 0001 makes jdlint the oracle until the port is complete. The xjd
repository already has the fixture (`fixtures/home`) and jdlint's expected
findings (`fixtures/expected-findings.json`).

- **Differential**: run `xd check --json` on that fixture and compare
  finding codes and paths with jdlint's file. Then generate random trees
  (proptest over small JD systems with injected faults: duplicate IDs, stray
  folders, empty categories) and compare both tools' findings, while jdlint
  is available in the dev shell.
- **One fixture per finding code**, the smallest tree that triggers it, so
  a regression names the rule.
- **JSON output is a contract** with the xjd mod: a schema test (and a
  snapshot) so a field rename is caught here, not in xjd.

### Plugins (WIT components)

- **Conformance suite** (`xd-conformance` crate): the laws above, exported
  as functions over a scheme or projection. Built-in schemes, Nickel
  configurations and each plugin run the same suite.
- **Type mismatch is rejected at load**: a component built against an older
  WIT world fails to instantiate, with a readable error. One test with a
  deliberately wrong component committed as a fixture.
- **Sandbox**: a plugin can't read outside what it's given (a component that
  tries to open a file fails).

### CLI end to end

Few, and only for what nothing else covers: argument parsing, exit codes,
`--json`, and the full `xd plan` / `xd apply --yes` / `xd undo` loop on a
temp root inside a temp jj repo. Use `assert_cmd` and snapshot the output.
Five to ten tests in total.

## Targets

- **Line coverage**: `xd-core` 90% or more, `xd-nickel` 80%, measured with
  `cargo llvm-cov` in CI and shown on the pull request, not enforced as a
  gate until the plan module exists. Not set up yet.
- **Mutation testing**: `cargo mutants` on `xd-core` weekly. The goal is no
  surviving mutant in `registry.rs` and the plan module: those hold the
  never-reuse and undo promises, where a test that runs the code but checks
  nothing is worst. Not set up yet; three hand-made mutants were checked
  (see "What's covered"), and one survived only because it was equivalent.
- **Property cases**: 512 for the laws, 256 for the registry, 128 for the
  Nickel agreement (each case evaluates a Nickel program), 200 for real
  folders; `PROPTEST_CASES` overrides all of them, and the weekly CI job
  sets 10,000. Seeds of real failures are committed next to each test file
  (`*.proptest-regressions`).
- **Speed**: `cargo test` under a minute; the folder tests stay at 200 cases.

## Not tested

- Formatting of Rust and Nickel beyond `cargo fmt --check` and
  `nickel format`.
- `serde` derives on their own; they are covered by `jd.ncl` loading into
  the Rust types.
- The docs. Examples in them that are code become doctests when the API
  settles.

## Next

The plan module, written with its table test, convergence and inversion
laws from the start, with its generators and laws added to
`xd-conformance` alongside today's.
