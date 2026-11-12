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

## Coverage today

| Area | Tested | Kind |
| --- | --- | --- |
| `Scheme::validate` | each error variant once | examples (`invalid_schemes_are_rejected`) |
| Render and parse numbers (JD, hex) | yes | examples |
| `parse(render(n)) = n` | bases 2–36, depth 1–2 levels | property |
| Plain string order = tree order | bases 2–36 | property |
| Real folders round trip | bases 2–64, `read_dir` | property on temp dirs |
| Custom alphabets, base64url's `-`, base58 | yes | examples |
| `portability()` | case collision, Windows chars | examples |
| `name::parse` / `render` | JD folder names | examples |
| Registry: allocate, retire, reuse, missing parent | JD system | examples |
| `Policy::LowestFree`, full category | yes | examples |
| `jd.ncl` equals `xd_core::jd::scheme()` | yes | differential |
| Nickel contracts reject; Rust validation backs them up | a few cases | examples |
| Open `System`, closed `Scheme` | yes | examples |

Everything passes (`cargo test`: 25 tests, 4 of them property tests, about a
second after the build).

## Gaps, most important first

### Bugs found while writing this

Two panics, both reproduced on this branch with a scratch test:

1. **`Address::to_integer` overflows.** `Scheme::validate` caps each level
   at 32 bits but not the sum, so base 64 with three 5-digit levels is
   valid, and `to_integer` on its largest address panics in debug builds
   (`attempt to multiply with overflow`, `address.rs:168`) and wraps in
   release. Fix: either validate that the whole address fits in 64 bits, or
   return `Option<u64>` / `u128`. Test: a property over any valid scheme
   that `to_integer` doesn't panic and is strictly increasing in tree order
   for one level.
2. **`Address::render` with another scheme panics.** An `Address` doesn't
   know its scheme, so rendering a 3-level address with a 2-level scheme
   indexes past `levels` (`address.rs:97`). It can't happen while there is
   one scheme per process, but it will with several systems loaded. Fix:
   make render return `Option`, or carry a scheme id in `Address`. Test: an
   example per mismatch.

### No CI

The repository has no `.github/workflows`. Nothing runs the checks in
`AGENTS.md` on a pull request. This is the first gap to close: a workflow
running `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`, `nickel format --check` and `nickel typecheck` on `nickel/*.ncl`,
and `nix flake check`, on Linux; plus `cargo test` on macOS, whose
case-insensitive disk is the one `portability()` warns about (see below).

### Core laws not yet tested

| Law | Why it matters | Test |
| --- | --- | --- |
| **GetPut**: `render(parse(name)) = name` for canonical names, and every other accepted name is reported as non-canonical | design-patterns.md adopts lenses; today `1a.1f` (hex) parses and renders as `1A.1F`, and `11.11  X` (two spaces) keeps a label starting with a space | property: generate canonical names, assert identity; generate variants (case, extra spaces), assert `parse` flags them once a `canonical` flag exists |
| **Registry invariants under any sequence of operations**: active ∩ retired = ∅; retired only grows; every active entry's parent is active; allocate never returns an active, retired or reserved number; `AfterHighest` never goes down | the whole of formal 1.4; today only a fixed script of operations checks it | stateful property test (proptest's state machine, or a vector of random `Op`s applied to a `Registry` and a naive model, compared after each step) |
| **Allocation fills a level exactly**: with no retirements, allocating until `Full` hands out every non-reserved number once | catches off-by-one at `capacity` | property over small schemes (base 2–4) so it finishes fast |
| **Top-level allocation for JD skips `00-09`** | areas are allocated too | example |
| **Parse never panics** | names come from disks and other people | fuzz `name::parse` and `Address::parse` with arbitrary UTF-8 (`cargo fuzz`, or a proptest over `any::<String>()` in the normal suite) |
| **Validated schemes are safe**: for every scheme `validate` accepts, render, parse, capacity and `to_integer` don't panic | the overflow above slipped through because the generator only builds up to 3 small levels | widen `any_scheme()`: bases 2–64 with alphabets, 1–6 levels, widths up to the 32-bit cap, separators drawn from a safe set |
| **Ranges**: `parse` rejects `10-18`, `10-29`, and bare area digits; accepts only a level marked `range` | the range branch has the most conditions | examples, then a property: `parse(s)` is `Some` only if `render(parse(s)) = s` modulo case |

### Nickel and Rust agree

`jd.ncl` vs `jd::scheme()` is checked, but the contracts in `xd.ncl` and
`Scheme::validate` are two definitions of "valid scheme". Add a
**differential property**: generate schemes (valid and slightly broken),
export each as Nickel, and assert that whatever `validate` accepts the
contract accepts too. The reverse direction (contract accepts, `validate`
rejects) is allowed and expected, and each such class should have a named
example like `rust_validation_catches_what_contracts_dont`. Also: a
misspelled field anywhere in `Level` is rejected, and error messages name
the field (a snapshot of the error text, so a regression in diagnostics is
seen).

### Disks other than Linux ext4

`folders.rs` proves round trips on the CI machine's file system only. The
real risk is case-insensitive disks (macOS APFS by default, Windows NTFS).
Run the folder tests on macOS in CI and add one that asserts: when
`portability()` reports a `CaseCollision`, creating both folders really
does collide there, and when it reports nothing, it doesn't. That turns the
portability warnings from documentation into a checked claim.

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
  gate until the plan module exists.
- **Mutation testing**: `cargo mutants` on `xd-core` weekly. The goal is no
  surviving mutant in `registry.rs` and the plan module: those hold the
  never-reuse and undo promises, where a test that runs the code but checks
  nothing is worst.
- **Property cases**: 256 by default, 10,000 in a weekly job; regressions
  committed under `proptest-regressions/`.
- **Speed**: `cargo test` under a minute; the folder tests stay at 200 cases.

## Not tested

- Formatting of Rust and Nickel beyond `cargo fmt --check` and
  `nickel format`.
- `serde` derives on their own; they are covered by `jd.ncl` loading into
  the Rust types.
- The docs. Examples in them that are code become doctests when the API
  settles.

## Suggested order

1. CI workflow (Linux and macOS) running what `AGENTS.md` lists.
2. Fix and pin the two panics above.
3. Registry state-machine property test and the wider scheme generator.
4. GetPut, with a `canonical` signal from `parse`.
5. Nickel/Rust differential property.
6. Then the plan module, written with its table test and inversion law
   from the start.
