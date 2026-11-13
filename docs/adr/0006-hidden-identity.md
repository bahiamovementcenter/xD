# 6. Hidden identity

Status: proposed, 2026-11-13.

## Context

Formal 1.3 makes the number an entry's identity and the label a name that
can change. Formal 1.5 requires that reading a folder name back gives the
identity: `parse(render(n)) = n`, a sealed rule in
[extensibility.md](../extensibility.md) ("folders must read back as what
was written").

PARA on its own (ADR 0005, `research/para-jd.md`) has no numbers. The
simplest identity for it is the item's name, but then a rename is a new
identity: links to the item break, and the index loses its history. Ash
asked (2026-11-13) for a hidden identity instead, made from a hash of the
title and/or content, configurable, and not tied to PARA.

[jd-experiments.md](../jd-experiments.md) already separates the questions:
E10 splits identity, label, pointer and proof, and lists a content hash as
"proof only" because it changes on every edit; E4 proposes a content digest
per entry as its version. ADR 0003 already hashes contents (SHA-256,
`sha256:` prefix) to check adopted files.

The options were compared in `research/identity-tags-placement.md`.

## Decisions

### 1. Each number space has an identity strategy

```nickel
identity = 'shown                              # today's rule, the default
identity = 'hidden { generate = …, store = … }
```

- **`'shown`**: the identity is in the folder name, rendered and parsed by
  the space's pattern (ADR 0004). JD and work packages keep this. Nothing
  changes for them.
- **`'hidden`**: the identity is not in the name. The name is the label
  alone, and the identity is generated (decision 2) and stored beside the
  entry (decision 4).

### 2. Generators, a closed set

| Generator | Identity is | Use |
| --- | --- | --- |
| `'sequence` | the next free number (formal 1.4) | JD, work packages; only with `'shown` |
| `'name` | the label itself; a rename is a move to a new identity (ADR 0003) | PARA alone when hidden IDs aren't wanted |
| `'ulid` | a ULID: time-ordered, random | flat spaces where creation order matters |
| `'hash { inputs, algorithm, length, alphabet }` | a hash of chosen inputs, taken once (decision 3) | Ash's choice for PARA; aliases (decision 7) |

xD owns every variant, so this is a closed enum (extensibility.md,
Strategies), named in Nickel.

### 3. A hash identity is taken once, at creation, and frozen

The hash is computed when the entry is created (the apply step that runs
its `CreateFolder` or `Adopt`) and never recomputed. Renames and edits don't
change it.

- **`inputs`** is any subset of `'title`, `'content`, `'created`, `'nonce`.
  `'content` is the entry's content digest at creation (decision 5).
- **The default is all four.** Without `'created` or `'nonce`, two items
  with the same inputs get the same identity (two empty folders called
  `Notes`). A space may leave both out only by declaring
  `content_addressed = true`: identical inputs then *are* the same entry, and
  a second one is reported as `duplicate-identity` rather than created.
- **`algorithm`** is `'sha256` (already a dependency, ADR 0003). The full
  hash is kept, prefixed `sha256:`, so the algorithm can change.
- **`length`** and **`alphabet`** give the short form used in links and
  shown to people: by default 10 characters from `alnum` (ADR 0004's
  case-folding 0-9A-Z, so it survives case-insensitive file systems).
- **Collisions** of the short form: allocation (formal 1.4) checks it against
  every live and retired identity of the space and regenerates with a new
  nonce. Retired identities are never reused, as for numbers.

### 4. Where a hidden identity is stored

The space's `store` is an ordered list; xD writes to the first that applies
to the entry and reads from all of them:

| Store | Where | Applies to |
| --- | --- | --- |
| `'marker` | a file `.xd.ncl` inside the entry's folder | folders |
| `'sidecar` | a file `<name>.xd.ncl` beside the entry | single files |
| `'suffix` | a visible suffix in the name, `Website redesign ·k3f9q2m1x0` | anything; makes the identity shown |

The default is `['marker, 'sidecar]`. Under the default entry form (`'folder`, ADR 0005 decision 8) every entry is a folder, so the marker is used; the sidecar matters only where a consumer allows single-file entries. The index (`entries.json`, ADR 0003)
always holds every identity as a cache, but is never the only copy.

The marker is a Nickel record, so it can be checked later:

```nickel
{
  id = "k3f9q2m1x0",
  hash = "sha256:…",
  created = "2026-11-13T23:07:52Z",
  generated = { by = 'hash, inputs = ['title, 'content, 'created, 'nonce] },
}
```

It is the per-folder metadata file the workflow rules call `.jd.ncl`,
renamed for xD; other per-folder fields go in the same file rather than a
second one.

The marker's file name is configurable, `store.marker = { file = ".xd.ncl" }`,
for hosts that forbid or hide leading dots differently. That has
consequences, each handled here:

- **`scan` needs the name before it walks.** It reads the system's
  configuration first (it already does, to find `.xd/`), so a marker is
  never mistaken for user content and adopted.
- **The name must be inert.** A contract checks it is a valid file name and
  that every pattern in the system parses it to `skip` (formal 1.5), so a
  marker can never be read as an entry.
- **Changing it is a plan.** Every marker is renamed by a `Rename` step, so
  the change can be checked and undone like any other (ADR 0002). Until the
  plan is applied, readers accept both names.
- **Nested systems** may use different names. Each reads only its own, and a
  folder holding another system's `.xd/` is that system's root, and `scan`
  should skip it as it skips its own `.xd/` (ADR 0003).
- **The format is not configurable.** It is always a Nickel record, so any
  host that embeds Nickel can read it.

Not adopted, for now:

- **Extended attributes**: many sync tools, archives and file systems drop
  them.
- **Metadata inside the file** (Markdown front matter, XMP): writing it
  changes the file's contents, which xD otherwise never does. Reading an
  identity that is already there is a later option.
- **The index alone**: a hand move would look like a delete and an add.
- **The file system's own ID** (inode, Windows file ID): lost on copy, sync
  or another machine. Used only as a hint (decision 6).

### 5. The content digest is a separate thing: proof, not identity

E4's digest (a Merkle hash of the entry's contents) is the entry's
*version*. It changes on every edit, is recomputed when needed, and is used
to detect changes, to sync, and to re-find entries (decision 6). It is never
an identity. A hash identity's `'content` input is the digest's value at
creation, frozen.

### 6. Re-finding an entry that lost its store

If `xd scan` finds a folder without its marker, or the index lists an
identity whose folder is gone, `check` looks for a match by content digest,
and on the same machine by file-system ID. One match: it proposes restoring
the marker (a plan step, so it can be undone). None, or more than one: it
reports `identity-lost` and guesses nothing.

### 7. A shown space may carry a hidden alias

E10's "each slot holds a set, with one primary" allows a JD entry to have a
hidden identity *as well as* its number:

```nickel
identity = 'shown, alias = 'hidden { generate = 'hash {…}, store = ['marker] }
```

The number stays primary. The alias survives a renumber, so it is what
something outside the system (a note, a link, another machine) should point
at. This is the second use of hidden identity besides PARA alone.

### 8. The sealed projection rule, reworded

extensibility.md's sealed row and formal 1.5 change from "`parse(render(n))
= n`, folders read back as what was written" to: **a projection is lossless
on the identity**. For a `'shown` space, the projection is the name, as
today. For a `'hidden` space it is the name together with the entry's store,
and reading both back gives the identity. The intent (what is written reads
back the same) is unchanged; the law in `xd-conformance` is extended to
round-trip the store.

## Consequences

- PARA can stand alone with identities that survive renames.
- JD gains optional stable aliases without touching its numbers.
- Every hidden-identity folder carries one hidden file, and every
  single-file entry one sidecar. Tools that show hidden files will show
  them.
- The core gains identity strategies on spaces, a marker reader and writer
  in the apply step (still the only I/O), and the `identity-lost` and
  `duplicate-identity` findings.
- Tests: the reworded projection law; that a hash identity is stable under
  rename and edit; that collisions regenerate; that a marker survives a move
  and is re-found by digest when deleted.

## Open

- Whether `'name` should also allow a later switch to `'hidden` (a
  migration plan that writes markers for every entry).
- Reading identities already present in file metadata (front matter, XMP).
