# 5. Facets and placement

Status: proposed, 2026-11-13.

## Context

ADR 0004 (decision 4) makes systems units that stack with `&`: JD, work
packages, and next PARA, each usable alone. PARA numbers nothing. Its rules
are about where things live: four buckets (Projects, Areas, Resources,
Archives), every item in exactly one, organized "by actionability"
([Forte Labs](https://fortelabs.com/blog/para/)), and moving between buckets
is routine. ADR 0004 left PARA's two needs to this record: a bucket facet
from a code list, and folder roots chosen by it.

xD already has a mechanism of the second kind, as a design: the four archive
strategies in [design/workflow-rules.md](../design/workflow-rules.md)
(`'standard`, `'last_slot`, `'in_place`, `'separate_root`). They decide where
an archived entry lives without changing its number, and "an area, category
or single entry can override" the system's choice. `'separate_root` already
puts entries under a different root folder. None of it is in the core yet.

By [extensibility.md](../extensibility.md), a mechanism waits for its second
real use. Archive strategies and PARA are two uses of the same thing: an
entry's place on disk, chosen from what is true of it, not from its number.
Ash named PARA + JD as xD's second reference configuration (not a default,
2026-11-13).

The design work is in the project's research notes:
`research/para-jd.md` (the units and how they stack) and
`research/identity-tags-placement.md` (library versus PARA defaults).

## Decisions

### 1. The library provides mechanisms; units are presets

Facets and placement are mechanisms with parameters in `nickel/xd.ncl`.
JD, work packages and PARA set those parameters as defaults
(`| default`), and a consumer's own merge overrides any of them. No unit
refers to another by name; a unit that binds to a neighbour checks for its
field (`std.record.has_field`), as `work-packages.ncl` does for `pattern`.

### 2. Facets: named fields on entries, from a code list

A facet is declared once per system:

```nickel
facets.para = xd.facet {
  values = { project = "Projects", area = "Areas", resource = "Resources" },
  cardinality = 'one,          # or 'many
  required = true,
  on = ['tree, 'work_packages],  # which number spaces may carry it
  inherit = 'down,             # or 'none
  default = fun entry => if entry.space == 'work_packages then 'project else null,
}
```

and set on entries as an ordinary field (`para = 'area`). Rules:

- **A facet is never identity.** It is metadata (formal 1.3): it is not in
  the number, not in the folder name, and changing it never renumbers.
- **`values`** is a code list: each value is an enum tag with a label. The
  label is for display and for placement (decision 4); renaming a label
  changes no entry.
- **`cardinality`** `'one` takes a tag, `'many` an array of tags without
  duplicates. Ordinary tags are `'many`, `'none` inheritance, not required;
  they need no further mechanism.
- **`required`** is checked on the effective value (decision 3).
- **`on`** lists the number spaces (ADR 0004) whose entries may carry it.
- **`archived` is not a facet.** It is an entry's state in its number space
  (formal 1.4), which every space has.

### 3. One inheritance rule for per-level settings

An entry's effective value for a facet, or for a placement setting, is:

1. its own value, if it has one;
2. otherwise its parent's effective value, if `inherit = 'down`, where the
   parent is formal 1.2's (drop the last component);
3. otherwise the facet's `default` (a value, or a function of the entry);
4. otherwise none.

Work packages have no parent, so they take their own value or the default.
This is the rule the archive strategy already describes ("an area, category
or single entry can override it"), now written once for both.

### 4. Placement: which root, and what layout inside it

```nickel
placement = xd.placement {
  roots = { main = ".", archive = "~/Archives" },
  root = fun entry => if entry.archived then 'archive else 'main,
  layout = fun entry => 'mirrored,
}
```

- **`roots`** names the top folders. `main` is the system's own folder.
- **`root`** maps an entry (its number, space, effective facets, state) to
  a root name. It is a Nickel function, so a unit can express any rule.
- **`layout`** places the entry inside its root, from a closed set
  (extensibility.md: xD owns every variant):

| Layout | Path inside the root | Example |
| --- | --- | --- |
| `'mirrored` | the entry's tree path: area, category, ID folders | `10-19 Life admin/11 Me/11.13 Taxes` |
| `'flat` | the entry's folder name alone | `W0012~11.13 2026 tax return` |
| `'into_slot` | inside the nearest reserved archive slot: `ak.09` for an ID, `a0.09` for a category, `00.09` for an area, `W0009` for a work package (needs the standard zeros) | `11.09 Archive/11.13 Taxes` |
| `'last_slot` | inside the `x9` category or the `90-99` area | `19 Archive/11.13 Taxes` |

Nickel evaluates `root` and `layout` for every entry and hands the core the
resolved values (a root name and a layout tag per entry), so the core never
runs a configuration's functions. The core's folder projection turns them
into a path and stays pure (ADR 0001).

### 5. Archive strategies become presets of placement

| Strategy | `root` for archived entries | `layout` for archived entries |
| --- | --- | --- |
| `'standard` (JD default) | `main` | `'into_slot` |
| `'last_slot` | `main` | `'last_slot` |
| `'in_place` | `main` | the live layout (nothing moves) |
| `'separate_root` | `archive` | `'mirrored` |

`archive = { strategy = …, root = … }` stays the way to write them, as sugar
that sets `placement`. Live entries use `main` and `'mirrored` unless a unit
says otherwise. Nothing a JD user writes today changes. `'in_place`'s cold
storage (`archive.storage`) is a storage concern, not placement, and is
unaffected.

### 6. Changing a facet or placement is a plan of moves

Retagging an area from `'resource` to `'area`, changing a root's path, or
changing the archive strategy changes effective paths. The planner emits
`Move` operations for every affected entry (ADR 0002 decision 2), which
keep their numbers (ADR 0003: a move is not a retirement) and undo like any
other move. The workflow rules already say this for strategy changes ("the
plan moves what's already archived to its new place").

### 7. Findings

| Code | When |
| --- | --- |
| `facet-value` | a value isn't in the facet's code list |
| `facet-required` | a required facet has no effective value |
| `facet-cardinality` | `'one` given an array, or `'many` with duplicates |
| `facet-space` | a facet set on an entry of a space not in `on` |
| `unknown-root` | `root` returns a name not in `roots` |
| `placement-collision` | two entries resolve to the same path |
| `slot-missing` | `'into_slot` or `'last_slot` where the system lacks the reserved numbers |

The first four and `unknown-root` are Nickel contracts, so they fail at
evaluation. `placement-collision` and `slot-missing` need every entry and
are checked by the core's `check` (ADR 0003).

### 8. PARA is a preset of both

`para.ncl` (not part of this record) will set:

- facet `para` as above: `'one`, required, `'down` from JD areas and
  overridable per category and ID, work packages defaulting to `'project`;
- roots `projects`, `areas`, `resources`, `archives`, with `root` from the
  `para` label, and archived entries to `archives`;
- layouts: `'mirrored` for the JD tree, `'flat` for work packages and for
  PARA alone.

PARA's default for its Archives is therefore `'separate_root` into one
`Archives/` root, each item keeping its path. The bucket an item came from
stays in its `para` tag, so un-archiving can put it back.

## Consequences

- PARA needs no change to identity, numbering or the formal model's
  hierarchy. "Explicit hierarchy" (a parent from data), which the survey had
  listed for PARA, stays deferred with LCC, IPC and ACM CCS 2012 as its only
  uses.
- Ordinary tags come with the same mechanism, for any system.
- The archive strategies get one implementation instead of four.
- The core gains resolved placement on entries and a projection that uses
  it; `xd-nickel`'s loader carries the resolved values. Both stay free of
  I/O.
- Tests: in `xd-conformance`, a law that projection of a placement is
  injective on live entries (else `placement-collision`), and that a
  placement change followed by its inverse plan restores every path; Nickel
  assertions for the contracts in `nickel/tests/`.

## Open

- Whether `layout` stays a function of the entry or becomes a per-root
  setting; a function is more general, a per-root setting easier to read.
- Whether a facet's label may also be rendered into folder names (a `P`
  prefix on project folders, say). Not needed by PARA.
- How `xd scan` reports a folder found under the wrong root (a hand move
  between buckets): propose retagging, or propose moving it back.
