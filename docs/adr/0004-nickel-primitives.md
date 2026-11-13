# 4. xD's primitives as a Nickel library, built on std

Status: proposed, 2026-11-13.

## Context

xD is "a library of composable primitives" (ADR 0001), but until now its
Nickel side, `nickel/xd.ncl`, only described the core's `Scheme`: a base,
one alphabet and a list of levels. The primitives existed only in the Rust
core, so a program had to link xD to read or write a number, and a
configuration couldn't say anything the `Scheme` struct couldn't hold.

Ash asked for a library of Nickel embed that exposes what xD needs: slots
that hold alphanumeric characters, separators and labels, which compose.
Nickel is already embedded everywhere this project needs it: in Rust
through `nickel-embed`, in Go through its C library, in Gleam through its
NIF. A library written *in Nickel* is therefore usable from all of them,
and from the `nickel` CLI, without a binding of its own. Ash also asked that
it be built with Nickel's standard library in mind.

The survey of other addressing systems (project files,
`research/addressable-systems.md`) and the PARA + JD design
(`research/para-jd.md`) were inputs: per-slot alphabets alone cover MSC,
IPC, ACM 1998 and Maidenhead, and PARA's projects can be JD work packages
linked to an ID.

## Decisions

### 1. Five primitives, in `nickel/xd.ncl`

| Primitive | Is | Built with |
| --- | --- | --- |
| **Alphabet** | the digits, lowest first, and whether letters fold case | `Array String`, `std.string.characters`, a `std.contract.from_validator` (two or more, unique, no `/` or whitespace) |
| **Slot** | a fixed number of digits from one alphabet, named; optionally written as the range of its children | a closed record contract (`std.string.NonEmpty`, `std.number.PosNat`) |
| **Text** | a separator of formal 3.1 (`sep.within` `.`, `sep.range` `-`, `sep.label` space, `sep.child` `+`, `sep.link` `~`, `sep.disk` `/`) or a `literal` such as the `W` of `W0011` | a closed record with a `role` tag |
| **Link** | a number of another pattern, after the last slot (`W0012~11.13`), optional or required | a record holding the target pattern |
| **Label** | the free text after the number (formal 1.3) | a tag |

A **pattern** is an array of these, so patterns compose with `@` and with
Nickel's merge, like any other data. Each slot is one level of the address,
from the top; an address of depth `k` is written as the parts up to its
`k`th slot.

Ready-made alphabets follow the core's: `decimal`, `hex`, `alnum` (the
core's default digits, case-folding), `base58`, `base62`, `base64url`.

### 2. `xd.pattern` compiles a pattern into contracts, functions and a scheme

`xd.pattern parts` checks the parts (at least one slot, at most one link,
right after the last slot, a label only last and after a separator, no
range on the last slot) and returns:

- `render`, `name`, `parse`, `write`: numbers to strings and back.
  `parse` returns plain data, `{ address, link, label }` or `null`, so it
  exports to JSON for any embedding host; `write` is its inverse.
- `Numeral` and `Name`: contracts, so `"11.11 Passport" | jd.Name` works
  like `std.string.Matches`. (`Number` is a reserved word.)
- `scheme`: the core's `Scheme`, for patterns the core can hold.

Parsing compiles one regex per depth with `std.string.find`, built once per
pattern, as std's performance notes advise. Reading a field through a
contract re-applies the contract, so `pattern` copies its parts out of
their contracts once.

### 3. The core stays the reference; Nickel must agree with it

The core isn't rewritten. A pattern lowers to a `Scheme` when it fits:
one alphabet, one range separator, nothing after the last slot but a space
and the label, no link. Text before the first slot lowers into the first
level's separator, so work packages (`W0011`) already fit. Anything else
fails to lower with a message naming what the core lacks, and still works
in Nickel.

`crates/xd-nickel/tests/primitives.rs` makes this a law: every generated
scheme, written as a pattern, lowers back to itself and renders, names and
parses exactly as the core does, on rendered names and on near-miss text.
`nickel/tests/primitives.ncl` holds examples as `std.test` assertions.

`jd.ncl` is now written with the primitives, keeps the same `scheme`, and
also exports its `pattern` (not exported to JSON) for embedders:
`(import "jd.ncl").pattern.parse "11.11 Passport"`.

### 4. Systems are units that stack with `&`

Ash's rule: JD alone, then JD plus work packages, then PARA on top, each a
unit of its own, and PARA must also work without JD. Nickel's merge already
stacks records, so a unit is a Nickel file that declares what it needs from
the units below it and adds its own fields:

```nickel
(import "jd.ncl") & (import "work-packages.ncl")   # & (import "para.ncl"), later
```

- `jd.ncl` defines `pattern` (JD's number) and `scheme`.
- `work-packages.ncl` declares `pattern` without defining it and adds
  `work_packages`: `W` and four digits, linked with `xd.link` to whatever
  `pattern` the system below defines. It is tested stacked on JD and on a
  hex system, and leaves the core's `scheme` unchanged.
- PARA is the next unit. It numbers nothing itself, so it needs no pattern
  primitive: it needs a bucket facet from a code list and folder roots
  chosen by it, which come with their own ADR. Stacked on work packages,
  its projects can be work packages; without JD, it still has its four
  buckets.

### 5. Deliberately left out

Each waits for its second use (docs/extensibility.md):

- Variable-width slots, repeats and choice. When they come, the pattern
  contract must check the parse rule from the survey: a variable-width slot
  is followed by a separator or by a slot with a disjoint alphabet. Choice
  compiles to one regex, not `std.contract.any_of`, which only checks the
  immediate part of a contract.
- Code lists (facets) and placement roots, for the PARA unit.
- Teaching the core per-slot alphabets, links and literals after the last
  slot. Until then those patterns are Nickel-only.
- Publishing `xd.ncl` as a Nickel package (`Nickel-pkg.ncl`), so hosts can
  `import xd` through nickel-embed's package support instead of an import
  path.

## Consequences

- Any host that embeds Nickel can render, parse and validate xD numbers
  with no xD code: the library is the Nickel file.
- Two implementations of the same laws, kept equal by a differential
  property test. Each case evaluates a Nickel program, so `nickel-lang-core`
  is built optimized in dev profiles to keep `cargo test` near a minute.
- `xd.ncl`'s contracts now use std's (`std.string.NonEmpty`,
  `std.array.NonEmpty`) instead of hand-written predicates.
