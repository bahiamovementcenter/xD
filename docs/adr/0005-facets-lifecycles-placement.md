# 5. Facets, lifecycles and placement

Status: proposed, 2026-11-13.

## Context

ADR 0004 (decision 4) makes systems units that stack with `&`: JD, work
packages, and next PARA, each usable alone. PARA numbers nothing. Its rules
are about where things live: four buckets (Projects, Areas, Resources,
Archives), every item in exactly one, organized "by actionability"
([Forte Labs](https://fortelabs.com/blog/para/)), and moving between buckets
is routine. ADR 0004 left PARA's needs to this record.

xD already has a related mechanism, as a design: the four archive strategies
in [design/workflow-rules.md](../design/workflow-rules.md) (`'standard`,
`'last_slot`, `'in_place`, `'separate_root`). They decide where an archived
entry lives without changing its number, and "an area, category or single
entry can override" the system's choice. None of it is in the core yet.

Taken apart, archiving is four separate things:

1. **A state change**: the entry goes from active to inactive.
2. **An effect on identity**: in JD the number is retired and never reused
   (formal 1.4). The effect belongs to that *transition*, not to a place.
3. **A move**: the path changes because placement depends on state. The
   four archive strategies are four placement rules for one state.
4. **Sometimes a storage change**: `'in_place` with cold storage moves the
   bytes elsewhere without changing the path.

Seen that way, Archives is not special: it is one value of a **state**, and
PARA's four buckets are four states of an actionability lifecycle (Ash,
2026-11-13, asked what archiving is once abstracted). So the mechanisms are
facets (what is true of an entry), lifecycles (which facet values an entry
moves between, and what each move does) and placement (where an entry lives,
given its facets). By [extensibility.md](../extensibility.md), each has two
real uses: JD's archiving and PARA. Ash named PARA + JD as xD's second
reference configuration (not a default).

The design work is in the project's research notes:
`research/para-jd.md` (the units and how they stack) and
`research/identity-tags-placement.md` (library versus PARA defaults).

## Decisions

### 1. The library provides mechanisms; units are presets

Facets, lifecycles, placement and entry form are mechanisms with parameters
in `nickel/xd.ncl`. JD, work packages and PARA set those parameters as
defaults (`| default`), and a consumer's own merge overrides any of them. No
unit refers to another by name; a unit that binds to a neighbour checks for
its field (`std.record.has_field`), as `work-packages.ncl` does for
`pattern`.

### 2. Facets: named fields on entries, from a code list

A facet is declared once per system:

```nickel
facets.topic = xd.facet {
  values = { health = "Health", money = "Money", travel = "Travel" },
  cardinality = 'many,             # or 'one
  required = false,
  on = ['tree, 'work_packages],    # which number spaces may carry it
  inherit = 'none,                 # or 'down
}
```

and set on entries as an ordinary field (`topic = ['money]`). Rules:

- **A facet is never identity.** It is metadata (formal 1.3): it is not in
  the number, not in the folder name, and changing it never renumbers.
- **`values`** is a code list: each value is an enum tag with a label. The
  label is for display and for placement (decision 5); renaming a label
  changes no entry.
- **`cardinality`** `'one` takes a tag, `'many` an array of tags without
  duplicates.
- **`required`** is checked on the effective value (decision 4).
- **`on`** lists the number spaces (ADR 0004) whose entries may carry it.
- **`default`** is a value, or a function of the entry.

Ordinary tags are `'many`, not required, not inherited, and need nothing
more.

### 3. Lifecycles: a state facet with transitions and effects

A lifecycle is a facet of cardinality `'one`, required, whose changes are
governed:

```nickel
lifecycle = xd.lifecycle {
  values = { live = "Live", archived = "Archive" },
  initial = 'live,
  transitions = [
    { from = 'live, to = 'archived, effects = ['retire] },
  ],
}
```

- **Each number space has exactly one lifecycle**, read as the entry's
  `state`. A space that declares none gets `{ 'live }` with no transitions.
- **Only declared transitions are allowed.** Changing `state` along any other
  edge is the finding `transition-not-allowed`.
- **Effects** are a closed set, run by the core when the transition is
  applied. The first is **`'retire`**: the entry's identity moves from Live
  to Retired (formal 1.4), so it is never handed out again. `'retire` is
  one-way: a retired entry has no transitions out, and bringing its content
  back is a new entry with `from` (as a JD renumber is), never the same
  identity made live again (`revive-retired`).
- **Moves are not effects.** Placement (decision 5) follows from the new
  state, so a transition's move is planned like any other path change
  (decision 7).

JD's lifecycle is the one above, for the tree and for work packages: what
JD calls archiving is the `live → archived` transition, and its `'retire`
effect is formal 1.4's "never reuse".

### 4. Stacked lifecycles refine states

A unit stacked on another may **split a state into substates**:

```nickel
# PARA, stacked on JD: 'live becomes three substates, 'archived is kept
lifecycle = xd.refine base.lifecycle {
  live = { project = "Projects", area = "Areas", resource = "Resources" },
  transitions = [
    { from = 'area, to = 'resource }, { from = 'resource, to = 'area },
    { from = 'project, to = 'archived }, { from = 'area, to = 'archived },
    { from = 'resource, to = 'archived },
  ],
}
```

- A transition between substates of one base state carries no base effects
  (area ↔ resource is just a move).
- A transition that crosses base states **inherits the base transition's
  effects** (project → archived retires, as JD's live → archived does), and
  must correspond to a transition the base allows. So a stacked unit can add
  structure but can't weaken the rules underneath it.
- A unit with nothing beneath it declares its lifecycle directly. PARA alone
  has four states and no `'retire`: its hidden identities (ADR 0006) are
  never reissued anyway, so an archived item can be brought back with the
  same identity.

### 5. One inheritance rule for per-level settings

An entry's effective value for a facet (the lifecycle's state included) or
for a placement setting is:

1. its own value, if it has one;
2. otherwise its parent's effective value, if the facet has
   `inherit = 'down`, where the parent is formal 1.2's (drop the last
   component);
3. otherwise the facet's `default`, or the lifecycle's `initial`;
4. otherwise none.

Work packages have no parent, so they take their own value or the default.
This is the rule the archive strategy already describes ("an area, category
or single entry can override it"), now written once for all of them.

### 6. Placement: which root, and what layout inside it

```nickel
placement = xd.placement {
  roots = { main = ".", archive = "~/Archives" },
  root = fun entry => if entry.state == 'archived then 'archive else 'main,
  layout = fun entry => 'mirrored,
}
```

- **`roots`** names the top folders. `main` is the system's own folder.
- **`root`** maps an entry (its number, space, state and other effective
  facets) to a root name. It is a Nickel function, so a unit can express any
  rule.
- **`layout`** places the entry inside its root, from a closed set
  (extensibility.md: xD owns every variant):

| Layout | Path inside the root | Example |
| --- | --- | --- |
| `'mirrored` | the entry's tree path: area, category, ID folders | `10-19 Life admin/11 Me/11.13 Taxes` |
| `'flat` | the entry's folder name alone | `W0012~11.13 2026 tax return` |
| `'into_slot` | inside the nearest reserved archive slot: `ak.09` for an ID, `a0.09` for a category, `00.09` for an area, `W0009` for a work package (needs the standard zeros) | `11.09 Archive/11.13 Taxes` |
| `'last_slot` | inside the `x9` category or the `90-99` area | `19 Archive/11.13 Taxes` |

Placement knows nothing about archiving: it maps facets to paths. Nickel
evaluates `root` and `layout` for every entry and hands the core the resolved
values (a root name and a layout tag per entry), so the core never runs a
configuration's functions. The core's folder projection turns them into a
path and stays pure (ADR 0001).

### 7. Archive strategies become placement presets for one state

| Strategy | `root` when archived | `layout` when archived |
| --- | --- | --- |
| `'standard` (JD default) | `main` | `'into_slot` |
| `'last_slot` | `main` | `'last_slot` |
| `'in_place` | `main` | the live layout (nothing moves) |
| `'separate_root` | `archive` | `'mirrored` |

`archive = { strategy = …, root = … }` stays the way to write them, as sugar
that sets `placement` for the `archived` state. Live entries use `main` and
`'mirrored` unless a unit says otherwise. Nothing a JD user writes today
changes.

**Storage is a separate axis.** Where an entry's bytes live (the disk, cold
storage through `archive.storage`, a backed folder) is not placement: an
`'in_place` entry in cold storage keeps its path. Storage may also be chosen
per state, but it is not decided here.

### 8. Changing a facet or placement is a plan of moves

A transition (project → archived), a retag (resource → area), a changed
root path or a changed strategy all change effective paths. The planner
emits `Move` operations for every affected entry (ADR 0002 decision 2),
plus the transition's effects. Moves keep their numbers (ADR 0003: a move is
not a retirement) and undo like any other move. A `'retire` effect is undone
only by undoing the plan that applied it, before anything else has been
allocated (ADR 0003's undo rules); it is never undone by a later transition.

### 9. Entry form: folder, file, or either

Each number space says what shape its entries take on disk:

```nickel
entry_form = 'folder                  # or 'file, or 'either
wrap = { name = 'file_stem }          # how a lone file becomes a folder
```

- **`'folder`**: every entry is a folder. A lone file placed as an entry
  (dropped into a PARA bucket, or adopted, ADR 0003) is wrapped: the plan
  creates a folder named by `wrap` and moves the file into it,
  `Lease 2026.pdf` → `Lease 2026/Lease 2026.pdf`. Wrapping is a planned
  `CreateFolder` and `Move`, so it is shown first and can be undone.
- **`'file`**: every entry is a single file; a folder found as an entry is
  reported, not changed.
- **`'either`**: both are accepted as they are.

`wrap.name` is `'file_stem` (the file's name without its extension) by
default; `'file_name` keeps the extension.

The library declares the mechanism; units set it with `| default`, and a
consumer's merge overrides it, per space and, through decision 5's
inheritance, per area, category or entry:

| Unit | Default |
| --- | --- |
| JD | `'folder`: every ID is a folder, as the JD docs describe |
| Work packages | `'folder` |
| PARA | `'folder`, so a lone file in a bucket is wrapped |

With `'folder` everywhere, a hidden identity (ADR 0006) always has a folder
to hold its marker; sidecars are only needed where a consumer chooses
`'file` or `'either`.

### 10. Findings

| Code | When |
| --- | --- |
| `facet-value` | a value isn't in the facet's code list |
| `facet-required` | a required facet has no effective value |
| `facet-cardinality` | `'one` given an array, or `'many` with duplicates |
| `facet-space` | a facet set on an entry of a space not in `on` |
| `transition-not-allowed` | a state change along an edge the lifecycle doesn't declare |
| `refinement-weakens` | a stacked lifecycle crosses base states where the base has no transition |
| `revive-retired` | a retired entry made live again instead of a new entry with `from` |
| `unknown-root` | `root` returns a name not in `roots` |
| `placement-collision` | two entries resolve to the same path |
| `slot-missing` | `'into_slot` or `'last_slot` where the system lacks the reserved numbers |
| `entry-form` | an entry of the wrong shape that wrapping can't fix (a folder in a `'file` space) |

`facet-*`, `refinement-weakens` and `unknown-root` are Nickel contracts, so
they fail at evaluation. The others compare the configuration with the last
applied state or need every entry, and are checked by the core's `check`
(ADR 0003).

### 11. PARA is a preset of all of this

`para.ncl` (not part of this record) will set:

- its lifecycle: four states, `'project`, `'area`, `'resource`, `'archived`
  (labelled Projects, Areas, Resources, Archives); stacked on JD or work
  packages, as a refinement of their `'live` (decision 4); alone, directly,
  with no `'retire`;
- the state inherited `'down` from JD areas, overridable per category and
  ID; work packages starting as `'project`;
- roots `projects`, `areas`, `resources`, `archives`, with `root` taken from
  the state's label;
- layouts: `'mirrored` for the JD tree, `'flat` for work packages and for
  PARA alone;
- `entry_form = 'folder` (decision 9).

PARA's Archives is therefore one `Archives/` root, each item keeping its
path: JD's `'separate_root` strategy, reached through PARA's own state rather
than as a special case.

## Consequences

- Archiving stops being a feature of its own: it is a lifecycle transition
  with a `'retire` effect, plus placement for the resulting state.
- PARA needs no change to identity, numbering or the formal model's
  hierarchy. "Explicit hierarchy" (a parent from data), which the survey had
  listed for PARA, stays deferred with LCC, IPC and ACM CCS 2012 as its only
  uses.
- Ordinary tags come with the same mechanism, for any system.
- The archive strategies get one implementation instead of four.
- The core gains resolved state and placement on entries, transition checks
  and the `'retire` effect; `xd-nickel`'s loader carries the resolved
  values. The core stays free of I/O.
- Tests: in `xd-conformance`, laws that placement is injective on live
  entries (else `placement-collision`); that a placement change followed by
  its inverse plan restores every path; that every refined transition
  crossing base states carries the base's effects; and that no transition
  leaves a retired state. Nickel assertions for the contracts in
  `nickel/tests/`.

## Open

- Other effects, once something needs them (`'freeze`: make read-only;
  `'notify`: tell an app). Each waits for a use.
- Storage per state (decision 7): whether cold storage becomes a property
  of a state, like placement.
- Whether `layout` stays a function of the entry or becomes a per-root
  setting; a function is more general, a per-root setting easier to read.
- How `xd scan` reports a folder found under the wrong root (a hand move
  between buckets): propose the matching transition, or propose moving it
  back.
