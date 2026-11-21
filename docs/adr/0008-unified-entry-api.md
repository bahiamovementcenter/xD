# 8. One entry API over every layer and projection

Status: proposed, 2026-11-20.

## Context

Ash asked (2026-11-20) whether xD can offer one API for a whole composed
system: Johnny.Decimal, work packages, PARA, and a GTD-style task workflow,
kept in folders, an Org JDex and task apps, run from the CLI, from Emacs or
by an agent.

The pieces exist, but at two levels with nothing between them:

- **Low level:** plan operations (`CreateFolder`, `Rename`, `Move`,
  `WriteJdexEntry`, `Adopt` …, ADR 0002 and 0003), applied only after
  `check`.
- **App level:** the adapter interface in
  [design/workflow-rules.md](../design/workflow-rules.md): `create`, `link`
  and `refresh`, with `remove` and `rename` proposed.

ADR 0002 mentions commands such as `xd new` and `xd retire` that "write the
config … and then plan as usual", but no record says what that set of
commands is. Meanwhile ADR 0004 to 0006 have reduced every layer to the same
few properties of an entry:

| Layer | What it adds to an entry |
| --- | --- |
| JD | an identity in a number space (shown), a label, a tree parent |
| Work packages | an identity in a second space, a link to a JD ID |
| PARA | a lifecycle state (`'project`, `'area`, `'resource`, `'archived`) and placement from it |
| GTD | a lifecycle state (`'todo`, `'next`, `'waiting`, `'someday`, `'done`), a context facet |
| PARA alone | a hidden identity (ADR 0006) |

So a single API over entries covers every layer without knowing any of them,
and the places entries are kept (folders, the Org JDex, an app) are
renderings of the same entry: formal 1.5's projections.

The Org side was surveyed in the project doc *Emacs + Org inventory for
composed decimal systems*: identity in `CUSTOM_ID` (formal 1.5), the hidden
identity in Org's `:ID:`, facets as properties or tags, lifecycle states as
TODO keywords.

## Decisions

### 1. The model is entries; layers only set their fields

An entry is:

| Field | From | Note |
| --- | --- | --- |
| `space` | ADR 0004 | which number space (`jd`, `wp`, `para` …) |
| `identity` | ADR 0004, 0006 | shown number or hidden identity |
| `label` | formal 1.3 | not identity |
| `parent` | formal 1.2 | derived from the identity in a tree space |
| `links` | ADR 0004 | e.g. `W0012 → 11.11` |
| `state` | ADR 0005 | the space's lifecycle |
| `facets` | ADR 0005 | effective values, after inheritance |
| `placement` | ADR 0005 | resolved root and layout |
| `form` | ADR 0005 | folder or file |

No layer adds a field of its own. A layer that needs one that isn't here
needs an ADR, as a change to this model.

### 2. Intents: a closed set of verbs, each compiling to a plan

| Intent | Arguments | Plans |
| --- | --- | --- |
| `new` | space, label, parent or link, facets | allocate (formal 1.4), `RecordInConfig`, `CreateFolder`, a write per projection |
| `rename` | identity, label | `Rename`, a write per projection |
| `renumber` | identity, new parent | `Move` with `from` (ADR 0003: not a retirement) |
| `set` | identity, facet, value | `RecordInConfig`, then the moves its placement implies (ADR 0005 decision 8) |
| `transition` | identity, state | the same, plus the transition's effects; refused off a declared edge (`transition-not-allowed`) |
| `link` / `unlink` | identity, target | `RecordInConfig`, a write per projection |
| `adopt` | path, identity | `Adopt` (ADR 0003) |
| `query` | a filter | nothing: read-only |

- **An intent is a pure function** `intent(model, args) -> Plan<Draft>`. It
  never touches the disk. Everything it plans goes through `check` and
  `apply` (ADR 0002 decision 4), so every intent is shown before it runs and
  can be undone.
- **Intents name entries by identity**, never by path or label.
- **Archiving is not a verb.** It is `transition` to the state whose edge
  carries `'retire` (ADR 0005 decision 3). Likewise moving a PARA item to
  Resources, or a GTD task to `next`.
- **Layer words are names for transitions, set in Nickel.** A lifecycle
  transition may carry a `verb` (`{ from = 'todo, to = 'next, verb = "next" }`,
  `{ from = 'live, to = 'archived, verb = "archive" }`), and the CLI accepts
  `xd archive 11.13` as `xd transition 11.13 archived`. A preset adds words
  without new Rust.
- The set is closed (extensibility.md, Strategies: xD owns every variant)
  and `#[non_exhaustive]`.

### 3. Query is a filter over effective values

`query` takes a conjunction of: space, state, facet values, linked to,
under (an identity's subtree), label text. It returns entries as JSON with
their effective values. It is for tools and agents. Org's agenda and
`org-ql`, or a task app, stay the human views: they read the projections.

### 4. Projections are where entries are kept, each lossless on identity

A projection renders entries to a place and reads them back. Two are
designed:

| Projection | Identity | Label | State, facets, links |
| --- | --- | --- | --- |
| Folders | the folder name (shown) or the marker (hidden, ADR 0006) | the name after the number | placement only |
| Org JDex | `CUSTOM_ID` (shown) or `:ID:` (hidden) | heading text after the number | properties `XD_<NAME>`; optionally tags or TODO keywords (decision 5) |

- **Each projection keeps the sealed rule**: reading back what it wrote gives
  the identity (formal 1.5, extensibility.md).
- **The configuration is the source of truth; projections are "current".**
  An edit made in Emacs is a disk edit (ADR 0002 decision 1): `xd plan`
  proposes recording it, never silently undoes it. A save that lands while
  apply runs is `jdex_changed` (workflow-rules.md).
- `WriteJdexEntry` becomes `WriteProjection { projection, identity, before,
  after }`, with the same inverse.
- **A `Projection` trait arrives with the Org projection**, its second
  implementation (extensibility.md: no trait before two). Until then the
  folder projection stays concrete. Task apps (Things) and a Markdown JDex
  are later implementations of the same trait; a WIT interface waits for an
  outside implementer.
- **The Org projection is Rust, in the core.** Reading and rendering Org
  are pure functions over text, so they stay testable without Emacs and the
  apply step keeps its journal, inverses and `jdex_changed` check.
- **Emacs is the test oracle for it.** A conformance test runs
  `emacs --batch` with only built-in `org-element` over what the Rust
  projection writes, and checks that Org itself reads back the same
  identities (`CUSTOM_ID`, `:ID:`), labels, TODO states, tags and
  properties. The test is skipped where Emacs isn't installed; CI's Nix
  shell provides it.

### 5. Org rendering is a parameter, set by presets

How an entry's fields appear in Org is configuration, with these defaults:

| Field | Default | Option |
| --- | --- | --- |
| Tree level | a heading level per tree level | one file per area |
| State | property `XD_STATE` | `org.todo = true`: the lifecycle's values become `#+TODO:` keywords (GTD) |
| A facet | property `XD_<FACET>` | `org.tag = true`: tags, for agenda filters (PARA, contexts) |
| Links | property `XD_LINK` and an Org link `[[#11.11]]` | none |
| Folder | property `DIR` | none |

The JD preset sets nothing. A PARA preset may set `org.tag = true` on its
state; a GTD preset sets `org.todo = true`. A consumer overrides either.

### 6. Surfaces

- **Rust:** the intents and `query` are public functions of the core.
- **CLI:** one command per intent (`xd new`, `xd set`, `xd transition`,
  `xd link` …, plus each `verb`), printing JSON and writing a plan for
  `xd check` and `xd apply` (ADR 0003 decision 1). `--apply` runs check and
  apply in one step after showing the plan.
- **Editors and agents:** call the CLI and read its JSON. An Emacs package
  never allocates a number or decides placement itself.

### 7. Findings

| Code | When |
| --- | --- |
| `unknown-identity` | an intent names an entry that doesn't exist |
| `verb-ambiguous` | a verb names transitions in more than one lifecycle the entry has |
| `projection-identity-missing` | an Org heading inside the system has no `CUSTOM_ID` or `:ID:` where one is required |
| `duplicate_number` | two headings, or a heading and a folder of another entry, carry one identity (workflow-rules.md) |

`transition-not-allowed`, `revive-retired` and the placement findings come
from ADR 0005 unchanged.

## Alternatives considered

**An Emacs worker as the Org projection** (proposed 2026-11-21): a batch
Emacs reading JSON lines on stdin and upserting headings with `org-id` and
`org-element`, run natively or in the browser through container2wasm. Not
chosen as the projection, because:

- it writes without the `before` value or a file-hash check, so apply can't
  invert it or raise `jdex_changed`, and a long-lived worker saves stale
  buffers over edits made in the meantime;
- it puts placement, TODO keywords and facet rendering in Elisp, where
  decision 5 makes them preset parameters resolved by the core;
- it makes Emacs a runtime dependency of every xD install; in the browser it
  is a download of hundreds of MB with slow typing (workflow-rules.md).

Its useful part, Emacs's own Org parser, is kept as the test oracle
(decision 4).

## Consequences

- Each layer is configuration: a new one (GTD, a small-business preset) adds
  lifecycles, facets and verbs in Nickel, and the API covers it.
- The CLI grows from plan-level commands to intent commands, all ending in
  the same check and apply.
- The Org JDex becomes a projection with its own conformance laws in
  `xd-conformance`: `read(render(entries))` returns the same identities,
  labels, states, facets and links; a heading-text edit reads back as a
  rename, not a new entry.
- No sealed rule changes.
- Intents depend on ADR 0005 and 0006, which are still proposed: `set`,
  `transition` and the Org state rendering wait for them; `new`, `rename`,
  `renumber`, `link` and `query` over JD and work packages can be built
  first.

## Order of work

1. Intents `new`, `rename`, `renumber`, `link` and `query` over JD and work
   packages, with the folder projection.
2. The Org projection and the `Projection` trait, with its laws.
3. `set` and `transition`, with ADR 0005 built.
4. Verbs, and the Org options (decision 5).
5. A thin Emacs package over the CLI.

## Open

- GTD is not yet a configuration in `nickel/examples/`, so it doesn't count
  as a use (ADR 0007). The GTD rows above are its expected shape; it counts
  once it is written.
- Whether the transition `verb` field needs a capability code in
  `nickel/capabilities.ncl`, or is part of ADR 0005's lifecycle.
- Whether `query` grows ordering and paging, or stays a filter.
- How a task app's own states (Things' Today, Anytime, Someday) map onto a
  lifecycle: by a table in the adapter, or by a preset.
