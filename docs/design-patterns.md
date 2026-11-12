# Design patterns for xD

Which established patterns fit xD, where each one goes, and which to leave
out. Written 2026-11-12, after ADR 0001 and [extensibility.md](extensibility.md).
Nothing here is decided until an ADR says so; the last section proposes
which ones deserve one.

## What xD has to do

The patterns are chosen against these jobs, all from the formal model and
the earlier design work:

1. Hold a **desired state** (Nickel configuration), compare it with the
   **observed state** (folders, JDex, apps) and change one to match the
   other, in either direction: a config edit or a disk edit.
2. Never lose identity: the number is the identity, and retired numbers are
   never reused (formal 1.3, 1.4).
3. Show every change before making it, and undo it after (plan, apply,
   undo; history in jj).
4. Let anyone add schemes, projections, checks, storage and apps without
   changing the core, cheapest mechanism first (extensibility.md).
5. Derive views (search, wiki, graph, Typst, marimo) without them becoming a
   second source of truth.
6. Later: share a system between people and machines.

## The core: how a run works

### Functional core, imperative shell

[Gary Bernhardt](https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell):
decisions are pure functions over values; a thin outer layer does the I/O.

**In xD:** already the rule (ADR 0001, decision 3; AGENTS.md). `xd-core` does
no I/O. The shell is: read config, read disk, call the core, apply. Keep it
that way: every new feature first asks "which part is a pure function?".

**Adopt: yes (already).**

### Reconciliation: desired, observed and last applied

A [Kubernetes controller](https://kubernetes.io/docs/concepts/architecture/controller/)
is "a non-terminating loop that regulates the state of a system": it
compares desired with observed state and acts to close the gap. Terraform's
[`plan`](https://developer.hashicorp.com/terraform/cli/commands/plan) does
the same once, on request. `kubectl apply` adds a third input, the
`last-applied-configuration`, so it can tell "the user removed this from the
config" from "someone added this by hand".

**In xD:** this is the shape of `xd plan`, and it settles a question the
earlier design left half-open. With only two inputs (config and disk), a
folder that is on disk but not in config is ambiguous: deleted from config,
or created by hand? The third input answers it, and the design already has
it: the `jd-applied` jj bookmark (per machine) marks the configuration as of
the last apply.

| Config vs last applied | Disk vs last applied | Meaning | Plan |
| --- | --- | --- | --- |
| changed | same | a config edit | change the disk |
| same | changed | a disk edit | record it in config (or flag it) |
| changed | changed, same way | both agree | nothing to do |
| changed | changed, differently | a conflict | stop and ask |

It is level-triggered: `xd plan` recomputes from the three states every
time, rather than replaying events, so a missed watcher event can't leave
it wrong.

**Adopt: yes, as the definition of plan.** ADR candidate.

### The plan is data, interpreted several ways

From functional programming (free monads,
[tagless final](https://okmij.org/ftp/tagless-final/)) and the Command
pattern: describe effects as values, and let separate interpreters run them.

**In xD:** a plan is a list of operations (`CreateFolder`, `Rename`,
`Retire`, `WriteJdexEntry`, `SetStorage` …) and nothing more. Interpreters:

| Interpreter | Does |
| --- | --- |
| show | prints the plan (`xd plan`) |
| check | validates it against the invariants before anything runs |
| apply | performs it through the storage and app backends, journaling each step |
| invert | produces the plan that undoes it (`xd undo`) |
| test | applies it to an in-memory tree, for property tests |

Because hooks receive and return plans (extensibility.md), a hook can't
bypass check or undo. Every operation must have an inverse, or be marked
irreversible and need confirmation; that rule is what makes undo complete.

**Adopt: yes.** It's the first thing to build after the registry. ADR
candidate together with reconciliation.

### Typestate: only checked plans can be applied

[Typestate in Rust](https://cliffle.com/blog/rust-typestate/): encode a
value's stage in its type so the compiler rejects out-of-order use.

**In xD:** `Plan<Draft>` → `check()` → `Plan<Checked>` → `apply()` →
`Applied` (which can produce its inverse). `apply` accepts only
`Plan<Checked>`, so skipping the invariant check is a compile error rather
than a review comment.

**Adopt: yes, small.** No ADR needed.

### Parse, don't validate

[Alexis King](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/):
turn unchecked input into a type that can only hold valid values, instead of
checking and passing the unchecked value on.

**In xD:** `Address::new` already does this. `Scheme` doesn't: it is a plain
struct with a separate `validate()`, so an unvalidated scheme can reach
`render`. Split it into `SchemeConfig` (what Nickel produces, public fields)
and `Scheme` (only from `SchemeConfig::parse`, private fields). This also
answers the open follow-up in extensibility.md about public fields.

**Adopt: yes, before the first release.**

## The data: what an entry is

### Entity-component-system (facets)

[ECS](https://bevy.org/learn/quick-start/getting-started/ecs/), from game
engines: an entity is only an identifier; data lives in components attached
to it; systems are functions over all entities with given components. New
kinds of data are new components, with no change to the entity type.

**In xD:** this fits the "lego" goal and experiment E10 closely. The entity
is the number (the identity). Label, pointer, digest, owners, storage,
archive state, agent setup and any plugin's own data are components
attached to it. A check, a view or a backend is a system that asks for the
components it needs ("every entry with `storage` and `digest`"). A plugin
adds a component type without touching `Entry`, which is how the system
stays extensible at the data level, not only the behaviour level. The
registry becomes a set of component tables keyed by address.

Cost: lookups by component instead of struct fields, and component types
need registering. A full ECS library isn't needed; tables keyed by
`Address` are enough at this size.

**Adopt: yes, as the shape of `Entry`.** ADR candidate. This is also the
data-side answer to the problem CGP addressed (extensibility.md).

### Composite

The Gang of Four tree pattern: leaves and branches share one interface.

**In xD:** the tree is computed from the digits (formal 1.2), so there's no
object tree to build. Walking it is an iterator over the registry in tree
order. **Not needed.**

### Cascade: nearest wins

CSS's cascade and CODEOWNERS: a setting applies to a subtree, and the most
specific one wins.

**In xD:** `.05` agent space (narrowest wins: `13.05`, then `10.05`, then
`00.05`), `.owners` (E11), storage and archive strategy (set on system,
area or category, inherited). One generic lookup, `nearest(address,
component)`, walks `address.ancestors()` and returns the first entry that
has the component. Every inherited setting then behaves the same way.

**Adopt: yes, one helper.**

## Syncing two representations

### Lenses: render and parse as a pair with laws

Bidirectional transformations
([Foster, Pierce et al., *Combinators for bidirectional tree transformations*](https://www.cis.upenn.edu/~bcpierce/papers/lenses-toplas-final.pdf);
the [Boomerang](https://www.cis.upenn.edu/~bcpierce/papers/boomerang.pdf)
language): a lens is a `get` from a source to a view and a `put` that writes
a changed view back into the source, obeying round-trip laws.

**In xD:** formal 1.5's projections are lenses, with two laws:

- *PutGet*: what you write is what you read: `parse(render(n)) = n`. The
  property tests already check this, for every base from 2 to 64.
- *GetPut*: reading a name and writing it back changes nothing:
  `render(parse(name)) = name`. Not tested yet, and it doesn't hold for
  every input: the default digits accept `1a` but write `1A`. Such a name
  isn't canonical, and the plan should show renaming it as a normalization
  rather than doing it silently. The test is "GetPut holds for canonical
  names, and every other accepted name is reported".

The same idea covers the harder case: a config edit and a disk edit to the
same entry. Treating each projection (folder name, JDex note, Org heading,
`.jd.ncl`) as a lens with tested laws is what makes "edit either side" safe.
It also gives plugin authors a precise contract: a projection plugin must
pass both laws in the conformance suite.

**Adopt: yes, as the projection contract.**

### Event sourcing and CQRS

[Event sourcing](https://martinfowler.com/eaaDev/EventSourcing.html) stores
every change as an event and derives the current state from them;
[CQRS](https://martinfowler.com/bliki/CQRS.html) separates the model that
writes from the models that read.

**In xD:** take the read/write split, not full event sourcing. The write side
is config + disk, reconciled by plan and apply. The read side is the derived
views (search index, wiki, graph, Typst, marimo), rebuilt from the write
side and kept in the XDG cache, which is the rule already agreed for them.
History is jj's operation log, which already records every change; xD
doesn't need its own event store. Reconciliation (above) stays
level-triggered, so it never depends on replaying events correctly.

**Adopt: the CQRS split, yes. Our own event store, no.**

## The edges: backends and checks

### Ports and adapters

[Hexagonal architecture](https://alistair.cockburn.us/hexagonal-architecture/):
the core defines the interfaces it needs (ports); each outside system gets
an adapter.

**In xD:** storage kinds (`'dir`, `'repo`, `'rclone` …), apps (Org,
Markdown), history (jj) and later the shared registry are adapters behind
ports that the plan interpreter calls. With the rule from extensibility.md:
a port is defined when its second adapter exists.

**Adopt: yes (already implied by ADR 0001, decision 4).**

### Specification: checks as composable rules

Evans and Fowler's [Specification](https://martinfowler.com/apsupp/spec.pdf):
a business rule as an object with `is_satisfied_by`, combinable with and,
or and not.

**In xD:** each jdlint rule becomes one specification over the registry and
disk, with its finding code, and a system's check set is a combination of
them, chosen in Nickel (JD's set by default). Nickel contracts are already
specifications for configuration; this extends the idea to the tree.

**Adopt: yes, for `xd check`.**

### Middleware for hooks

[tower's `Layer`](https://docs.rs/tower/latest/tower/trait.Layer.html) wraps
a service in another, giving an ordered pipeline.

**In xD:** hooks around plan and apply are an ordered list of plan → plan
transforms (mdBook's preprocessors work the same way). An ordered `Vec` of
hooks is enough; a tower-style generic stack is more machinery than this
needs.

**Adopt: the idea, without tower.**

### Capabilities for plugins

Object capabilities: code can only use what it was handed. WASI is built
this way.

**In xD:** a plugin gets handles to what its manifest declares (a subtree,
read-only; no network) and nothing else. External-command hooks get the
nearest equivalent through Landlock. A plugin never gets "the file system".

**Adopt: yes (already in ADR 0001, decision 5).**

## Later: shared systems

- **CRDTs** ([crdt.tech](https://crdt.tech/)) for shared state that people
  edit at the same time: the JDex text and labels. They don't fit number
  allocation, which needs one serialized decision, so that stays a single
  allocator (the SpacetimeDB-style reducer in ADR 0001, decision 7).
- **Saga with compensations** for apply across several storage backends: if
  step 3 of 5 fails, run the inverses of steps 2 and 1. The plan's inverse
  operations already provide the compensations, so this costs little once
  plans are data.

## Leave out

- **Inheritance hierarchies**: Rust doesn't have them, and ECS components
  cover what subclassing would.
- **A global plugin registry or service locator**: hidden dependencies.
  Backends are passed in explicitly, which also keeps the core testable.
- **Full event sourcing**: jj already keeps history, and level-triggered
  reconciliation is more robust than replay.
- **CGP and tagless final in Rust**: compile-time generality we don't need
  yet (extensibility.md).

## Case study: Litho (deepwiki-rs)

[An article on Litho's plugin architecture](https://dev.to/sopaco/from-rust-to-multi-language-lithos-plugin-based-extensible-architecture-design-b8b)
describes plugin traits (language processors, LLM providers, output
adapters), a registry that discovers plugins from `./plugins` and the
environment, Rust `cdylib` plugins, lazy loading through `Box<dyn Any>`
downcasts, and an in-process "sandbox".

The [repository](https://github.com/sopaco/deepwiki-rs) (checked
2026-11-12) has the `LanguageProcessor` trait, but no registry, discovery,
dynamic loading or sandbox: a `LanguageProcessorManager` holds a fixed
`Vec<Box<dyn LanguageProcessor>>` of built-in processors and picks one by
file extension.

What xD takes from it:

- **The repository is the better design.** Built-in strategies chosen by
  data (a file extension there, a Nickel field here) are rung 3 of the
  ladder: cheap, typed and testable. It's where most xD extension points
  should start.
- **One output model for many inputs.** Every processor returns the same
  `CodeInsight`; every xD projection or app adapter reads and writes the
  same entries and components. Plugins vary the parsing, never the model.
- **What to avoid from the article:** Rust `cdylib` plugins (no stable Rust
  ABI, same process, unsafe to load), `Box<dyn Any>` downcasts (types lost at
  the boundary, the opposite of what xD wants), discovery from the working
  directory (a hidden dependency and a way to run unexpected code), and an
  in-process sandbox (code in the same process isn't isolated). WIT
  components cover the same needs with load-time type checks and real
  isolation.

## Proposed next ADRs

1. **Plan and apply** (written: [ADR 0002](adr/0002-plan-and-apply.md)). Plans are data with inverse operations; three-way
   reconciliation against the `jd-applied` bookmark defines the plan;
   typestate enforces check before apply.
2. **Entries as components.** The number is the entity; everything else is
   a component; inherited settings use one nearest-wins lookup.
3. **Projections are lenses.** Both round-trip laws are part of the
   conformance suite every projection must pass.

Smaller changes that don't need an ADR: split `SchemeConfig` from `Scheme`,
and the `nearest()` helper.
