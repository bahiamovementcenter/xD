# Designing xD for extensibility

How xD applies Microsoft's
[Designing for Extensibility](https://learn.microsoft.com/en-us/dotnet/standard/design-guidelines/designing-for-extensibility)
guidelines (from *Framework Design Guidelines*, 2nd edition, Cwalina and
Abrams). Microsoft marks the pages as not current, and they are written for
.NET classes. xD has no inheritance: it extends through Nickel
configuration, Rust traits and WIT components (ADR 0001). So each guideline
is restated in its own words below, then translated. Read the originals for
the full text.

## The rule behind all of them

> Choose the least costly mechanism that meets the need. You can add
> extensibility later; you can't take it away without breaking someone.

In xD the mechanisms, cheapest first:

| Rung | Mechanism | Who uses it | Cost to xD |
| --- | --- | --- | --- |
| 1 | **Nickel values**: change a field (`scheme.base = 16`) | anyone | none: already validated |
| 2 | **Nickel merges and contracts**: build on `jd.ncl`, add your own checks | anyone | small: contracts are data |
| 3 | **Built-in options**: pick a variant xD ships (`Policy::LowestFree`) | anyone | a variant to test |
| 4 | **Rust traits**: implement `Reserved`, later storage or app backends | Rust programs using xD as a library | an API to keep stable |
| 5 | **WIT components**: a wasm plugin | anyone, any language | an interface, a sandbox, versioning |

A new extension point starts on the lowest rung that covers it. A need that
only one person has stays on rung 1 or 2 until a second one appears.

## Guideline by guideline

### Unsealed classes: open by default

**Guideline:** let people derive from a type even without designing special
hooks; they add convenience members and defaults, and it costs little.

**In xD:** the equivalent is open data. A Nickel record can be merged with
another, so `(import "jd.ncl") & { ... }` is "deriving" a system from JD. The
top-level `System` contract is therefore open (`..`): a system can carry
fields of its own that xD ignores. `Scheme` and `Level` stay closed, so a
misspelled field (`seperator`) is an error instead of being silently ignored.
That is a reason to seal (see Sealing), and it's the only one used so far.

### Protected members: whatever an extension can see is public

**Guideline:** members visible only to subclasses still count as public for
security, documentation and compatibility.

**In xD:** Rust's `pub(crate)` is truly private, so it doesn't apply there.
It does apply to WIT: every function and type a plugin interface exports, and
every host function a plugin may call, is public API. It is documented,
versioned and treated as untrusted input. Advanced hooks go in a separate WIT
interface, so the common one stays small.

### Events and callbacks: let users run code at chosen moments

**Guideline:** callbacks and events let users customize behaviour without
understanding the whole design; avoid them on hot paths, and remember that a
callback runs arbitrary code.

**In xD:**

- `Reserved` is already a callback: any `Fn(&Address) -> bool` works, as
  `jd::standard_zeros` shows.
- Plan and apply are the natural events: before a plan is applied, after
  each change, after apply. A hook may inspect a plan or veto it, but changes
  go through the plan, so `undo` stays possible.
- Hot paths (parsing every folder name, rendering every number) never call a
  plugin per item. A plugin gets a batch.
- A callback is arbitrary code, so plugins run as WASI components with only
  the files and hosts their manifest grants.

### Virtual members: overridable, but the invariants stay put

**Guideline:** overridable members are costly to design, test and keep
compatible; limit them to what is necessary, and have public members call
the overridable part rather than making the public member overridable.

**In xD:** a trait's default methods are its virtual members, so a trait
gets as few as possible. The second half of the guideline is the important
one: the public operation stays in the core, and it calls the extension only
for the part that varies. `Registry::allocate` is the model. A policy or a
reserved-number callback chooses *among free numbers*; the core still
guarantees that a live, retired or reserved number is never handed out.
Every extension point is designed the same way: the extension decides the
variable part, the core enforces the formal model's rules around it.

### Abstractions: only with several implementations and reference tests

**Guideline:** an abstraction needs exactly the right members, and that is
hard; don't add one until several concrete implementations and consumers
have tested it; choose carefully between the two forms; ship tests that
implementers can run.

**In xD:**

- **No trait without two implementations and a consumer.** `Reserved` has
  two (`NoneReserved`, `jd::standard_zeros`). Storage, apps, history and
  archive get their trait or WIT interface when their second backend is
  written, not before.
- **Enum or trait.** When xD owns every variant, it's a closed enum
  (`Policy`): exhaustive and cheap to test. When outsiders implement it, it's
  a trait or a WIT interface. An enum can become a trait later; going back
  would break implementers.
- **Reference tests.** The property tests are the contract in executable
  form: `parse(render(n)) = n`, names read back from real folders, byte order
  equals tree order where the scheme promises it. They will move into a
  conformance suite that any scheme, projection or plugin must pass, with
  jdlint's findings as the suite for checks (ADR 0001, decision 6).

### Base classes: helpers for implementers, kept out of the way

**Guideline:** classes that hold shared implementation make abstractions
easier to implement, but add surface; keep them out of the main namespace
and don't name them `…Base`.

**In xD:** the helpers are default trait methods, ready-made values
(`alphabets::BASE58`, `jd.ncl`) and, later, a plugin SDK crate that fills in
the WIT boilerplate. They live in their own modules or crates (`alphabets`,
`jd`, a future `xd-plugin`), not in the core types, and they are named for
what they are.

### Sealing: seal only for a reason, and say which

**Guideline:** don't prevent extension just because no use comes to mind;
seal when there is a concrete reason.

**In xD**, these are sealed, each for a stated reason:

| Sealed | Reason |
| --- | --- |
| The number is the identity; the label is not (formal 1.3) | Everything else (renames, links, history) depends on it |
| Retired numbers are never reused (formal 1.4) | The JDex as a permanent record depends on it |
| `parse(render(n)) = n` for every scheme (formal 1.5) | Folders must read back as what was written |
| The core does no I/O; only apply touches the disk, reversibly | Plans must be safe to compute and to undo |
| `Scheme` and `Level` reject unknown fields | Typos must fail loudly |

Anything not in this table is open to extension or will be when its second
use appears. Adding a row to it needs an ADR.

## Considered: context-generic programming (CGP)

[CGP](https://contextgeneric.dev/blog/extensible-datatypes-part-1/) brings
extensible records and variants to Rust: modules each build part of a struct
without knowing the whole (extensible builders), or each handle some variants
of an enum (extensible visitors), with safe upcasting and downcasting between
enums that share variants.

Not adopted, as of 2026-11-12:

- It works at compile time, for Rust code built against xD (rung 4). Most
  extension happens in Nickel or WIT (rungs 1, 2 and 5), where it doesn't
  reach.
- Extensible builders are what Nickel merges already give every user, at
  configuration time, with no recompiling.
- It adds many macros and traits, longer builds and harder errors, and it is
  young (v0.4.2 in the post, v0.7.0 announced since).
- No abstraction before a second implementation, and there isn't one yet.

Revisit when an enum gets several independent handlers. The likely
candidates are folder entry kinds (`+` children, dot entries, work packages)
and check findings, once jdlint's rules are ported. Until then those enums are
`#[non_exhaustive]` and handled in one place.

## Follow-ups

- Mark public error enums `#[non_exhaustive]` before the first release, so a
  new error isn't a breaking change.
- Decide whether `Scheme`'s public fields become private with a builder
  before the first release; public fields can't grow without breaking
  struct literals.
