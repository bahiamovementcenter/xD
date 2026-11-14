# 8. Locations: where else an entry lives, and adapters for Git and forges

Status: proposed, 2026-11-14.

## Context

Ash asked (2026-11-14) how xD would integrate Git, remote repositories and
forges such as GitHub: as a plugin, an extension, or just another folder,
whether that needs a new convention, and whether the convention would have
to be built into the core.

The use that prompted it is Ash's own: a Johnny.Decimal system run across
Claude projects, Git and GitHub, with xD as a consumer. In it a project is
one JD ID, a work package is a Git branch whose pull request closes it, and
a GitHub repository belongs to an ID through a topic (`jd-001-05`) and its
description. That layout is Ash's and stays in Ash's own configuration, not
in xD's defaults. This record decides only what xD must provide so that a
layout like it can be written.

What xD already has:

- **Patterns render an identity into any string** (ADR 0004). A folder name,
  a branch name and a topic are all render targets, and `parse` reads each
  back.
- **Placement** (ADR 0005 decision 6) maps an entry's facets to one path on
  disk: a root and a layout. It has no notion of a second place.
- **Storage is a separate axis** (ADR 0005 decision 7): where the bytes
  live, not where the entry is named.
- **App adapters are a seam** (ADR 0001 decision 4), and the plan's
  "current" input already includes "the apps" (ADR 0002 decision 1). Neither
  is designed yet.
- **A hidden identity can be stored beside an entry** in a marker
  (ADR 0006 decision 4), and a shown space may carry an alias that outside
  things point at (decision 7).

What it lacks is a way to say "this entry also lives *there*": a repository,
a branch, a topic on a forge.

## Decisions

### 1. Naming on Git and forges is a pattern, not a new convention

A branch name, a repository name and a topic are projections of an
identity, exactly as a folder name is. They are written with the existing
primitives, so the naming needs no new convention and no core change:

```nickel
# a sketch: work package 12 on ID 001.05 as a branch, w0012-001.05-short-label
branch = xd.pattern ([literal "w", slot { name = "wp", width = 4 }, dash]
                     @ id_parts @ [dash, slug_label])
```

Each place has rules of its own about characters, and those are what the
units below supply:

| Target | Allowed | Consequence |
| --- | --- | --- |
| Git ref (branch, tag) | most characters, but not `~ ^ : ? * [ \`, space, `..`, or a trailing `.lock` (`git check-ref-format`) | JD's link separator `~` can't be used: `w0012~001.05-label` is not a valid branch name |
| GitHub repository name | letters, digits, `-`, `_`, `.` | no spaces in labels |
| GitHub topic | lowercase letters, digits, `-`; at most 50 characters; starts with a letter or digit | `001.05` becomes `001-05`; `W` becomes `w` |

So the units derive each place's pattern from the space's own by swapping
its text parts (patterns are arrays, ADR 0004 decision 1, so this is
`std.array.map`). The default swap is `-` for the link and within
separators, and a lowercased, hyphenated label: `-` is the only punctuation
all three targets accept. A consumer overrides it per place.

The swap happens only where a place needs it. `~` stays JD's link
separator (formal 3.1, `sep.link` in ADR 0004) on disk and in the JDex; a
branch is just another rendering of the same number, and `parse` on that
rendering reads `-` back as the link.

### 1a. Target rules are a reviewed registry

Rules like these are scattered, change rarely but without notice, and are
easy to get wrong from memory. So they are kept as data, the way Apple's
[password-manager-resources](https://github.com/apple/password-manager-resources)
keeps each website's password rules and quirks: one reviewed entry per
target, each citing its source.

`nickel/targets.ncl` holds, per target (`git_ref`, `github_repo`,
`github_topic`, and the file systems placement already writes to:
`posix`, `macos`, `windows`):

- the characters allowed, and those allowed only in some positions
  (a ref may not start with `-` or `.`, or end with `.lock`);
- case: preserved, folded on comparison, or forced lower;
- length limits;
- reserved names (Windows' `CON`, `NUL` …);
- `source`: the documentation or command the rule comes from
  (`git check-ref-format`, GitHub's docs, Microsoft's naming rules);
- `checked`: the date someone last confirmed it.

The name contracts (`GitRefName`, `GitHubTopic` …) are built from these
entries, so fixing a rule is a data change with no code. A quirk found in
use (a host that rejects something its docs allow) is added as an entry
with its evidence, as Apple's repository takes quirks. The entries are
plain data, so a tool outside xD can read them too.

The label in such a name is a slug, so writing it back loses the label's
spelling. That is allowed: the sealed rule is that a projection is
lossless *on the identity* (extensibility.md, ADR 0006 decision 8), and the
label is not the identity (formal 1.3). The rule applies to every location
name unchanged, and the round-trip law covers them.

### 2. Core gains one primitive: a location

A number space declares the **locations** its entries have. Each is named,
bound to an adapter (decision 4), and says how the entry's identity is
carried there:

```nickel
locations = {
  folder = xd.location.folder,                     # what placement resolves
  repo = forge.repo {
    host = 'github,
    carries = 'marker { topic = topic_pattern },   # the ID is in a topic
    cardinality = 'many,                           # an ID may own several repos
  },
  branch = git.branch {
    carries = 'name branch_pattern,                # the ID is in the name
    repo = 'link 'repo,                            # the linked ID's repo
  },
}
```

- **`folder` is the first location**, and every space has it. It is what
  placement (ADR 0005 decision 6) resolves. So the primitive has two kinds
  of implementation from the start, folders and remote places, and placement
  becomes the folder location's rule rather than a separate idea.
- **`carries`** is how a place reads back as the entry, the two ways ADR
  0006 already names: `'name` (the identity is in the place's name,
  rendered by a pattern; branches, tags, folders) or `'marker` (stored in
  metadata the place holds; a topic, a description prefix, a file in the
  repository). A repository whose name isn't derived from its ID, such as
  `xD` for `001.05`, is carried by a marker.
- **`cardinality`** is `'one` or `'many`, as for facets (ADR 0005
  decision 2). A required location without a binding is `location-unbound`.
- **`repo = 'link 'repo`** lets a place be found through a link: a work
  package's branch lives in the repository of the ID it links to
  (`W0012~11.13`, ADR 0004). Nothing else in xD names another space, so
  this is the only cross-space reference.
- **Which places are bound** is data, like entries: for `'name` places it is
  computed; for `'marker` places it is listed on the entry, or found by
  reading markers (decision 6).

The core never runs a configuration's functions: Nickel evaluates each
location's name and hands the core resolved values, as it does for
placement. The core knows a location's name, kind and adapter tag, and
nothing about Git.

### 3. A location's condition follows the entry's state

ADR 0005 keeps effects a closed set (`'retire`) and makes moves follow from
placement rather than from transitions. Locations work the same way. A
location says what condition it should be in for each state, and the plan
makes it so:

```nickel
repo = forge.repo {
  …,
  condition = fun entry => if entry.state == 'archived then 'archived else 'active,
}
```

So "retire → archive the repository" is not a new effect. Archiving the
entry changes its state; the repository location's condition for that
state is `'archived`; the planner emits the operation that archives it,
level-triggered like every other plan (ADR 0002 decision 1). Each adapter
lists the conditions it supports (decision 4); a condition it doesn't know
is a contract failure.

### 4. Adapters: a closed set first, in units of their own

An adapter knows one kind of place: how to address it, how to read its
current state, which plan operations it can carry out, and which
conditions it supports.

| Adapter | Places | Reads | Writes |
| --- | --- | --- | --- |
| `'folder` | folders under a root | the disk (ADR 0003) | the existing operations |
| `'git` | refs in a repository: branches, tags | refs, through any Git remote | create, rename, delete a ref |
| `'forge { host }` | repositories and their metadata | topics, description, archived flag | set topics and description, archive, unarchive |

- **Git and forges are separate adapters.** `'git` works over the Git
  protocol with any remote. `'forge` is per host and covers what Git
  doesn't (topics, archiving, pull requests). A consumer uses either alone.
- **They are shipped as units**, `nickel/git.ncl` and `nickel/forge.ncl`,
  stacked with `&` like JD, work packages and PARA (ADR 0004 decision 4).
  Each declares its location kinds, its name contracts (`GitRefName`,
  `GitHubTopic`, `GitHubRepoName`) and its default patterns with `| default`,
  so a consumer overrides any of them. Neither refers to JD; each checks for
  the fields it binds to (`std.record.has_field`), as `work-packages.ncl`
  does for `pattern`.
- **Rung 3 first** (extensibility.md): xD owns these variants, so they are
  a closed enum named in Nickel. The Rust side is ADR 0001's app-adapter
  seam, and it becomes a trait only when a second forge backend (Forgejo,
  GitLab) is written, and a WIT interface only when someone outside xD
  writes one.
- **`'forge` starts with `host = 'github`.** Other hosts are later variants.
- **An adapter may be read-only.** Its places are compared and drift is
  reported, but nothing is written. This covers places with no API to write
  to, such as a Claude project's name, without xD knowing what they are: a
  consumer's own tool can read them and pass them in as current state.

### 5. Plan operations for remote places

The plan gains operations for locations, each with its inverse
(ADR 0002 decision 2):

| Operation | Inverse |
| --- | --- |
| `BindLocation { entry, location, place }` | `UnbindLocation` |
| `RenameLocation { entry, location, from, to }` (a branch, a repository) | `RenameLocation` back |
| `SetLocationMeta { entry, location, before, after }` (topics, description) | the same, swapped |
| `SetCondition { entry, location, before, after }` (archive a repository) | the same, swapped |
| `CreateRef { entry, location, name, at }` | `DeleteRef { …, at }` |
| `DeleteRef { entry, location, name, at }` | `CreateRef` at the recorded commit |

- **xD doesn't create or delete repositories.** Deleting one can't be
  undone, and creating one is only undone by deleting it. A repository is
  bound once it exists; making it stays with the user or the forge.
- **Remote operations are listed apart** in the plan, because other people
  can see them as soon as they run. They are still reversible, so they need
  no confirmation beyond the plan's own, but `xd apply` names them
  separately.
- **A deleted ref keeps its commit** in the operation (`at`), so undo can
  recreate it. If the commit is gone from the remote by then, undo reports
  it instead of guessing.
- The core stays pure: adapters read current state before planning and
  write only during apply, as the folder backend does.

### 6. Drift from a remote place is a proposal, never an overwrite

Three-way reconciliation (ADR 0002 decision 1) applies to locations
unchanged, with the adapter's reading as "current":

- **A repository archived by hand** on the forge: propose the matching
  state transition (to `'archived`) if the lifecycle allows it, else report
  `transition-not-allowed`. This is the answer ADR 0005 left open for a
  folder moved by hand between roots, applied to locations too.
- **A branch renamed upstream**: propose recording the new name, or renaming
  it back.
- **A repository carrying an xD marker** for an identity the system doesn't
  have, or has retired: report `location-orphan`; propose nothing.
- **A pull request merged and its branch deleted**: the work package's
  branch location is gone; propose the transition the consumer's lifecycle
  ties to that (for work packages, usually to `'archived`). xD doesn't read
  pull requests themselves until a second use needs them.

### 7. Findings

| Code | When |
| --- | --- |
| `location-name` | a location's rendered name breaks its target's rules (decisions 1 and 1a) |
| `location-collision` | two entries resolve to the same place |
| `location-unbound` | a required location has no binding |
| `location-orphan` | a place carries the marker of an identity that is unknown or retired |
| `location-condition` | an adapter is asked for a condition it doesn't support |
| `location-read-only` | a plan would write to a read-only adapter's place |

`location-name` and `location-condition` are Nickel contracts, so they fail
at evaluation. The others need every entry or the current state, and are
checked by the core's `check` (ADR 0003).

### 8. Two consumers before building

`locations` is registered in `nickel/capabilities.ncl` as `'proposed`, and
`System.locations` reports it (ADR 0007 decision 1). Its first consumer is
Ash's JD across Claude projects, Git and GitHub, once it is written as a
config under `nickel/examples/`; until then it is a candidate in `known`,
like the others. A second config, such as a team's set of repositories per
ID, makes CI say it's time to build.

## Consequences

- Git and forges need no change to identity, numbering or the sealed rules.
  The round-trip law gains location names as projections.
- Placement becomes the rule of the built-in `folder` location; nothing a
  user writes for placement or archive strategies changes.
- The core gains resolved locations on entries, the operations of
  decision 5 and the findings of decision 7. It stays free of I/O and
  knows no adapter beyond its tag.
- `nickel/git.ncl` and `nickel/forge.ncl` are new units; a JD user who
  doesn't import them sees nothing new.
- Tests: in `xd-conformance`, laws that every location name round-trips the
  identity; that location names are injective across live entries (else
  `location-collision`); that a plan of remote operations followed by its
  inverse restores every location. Nickel assertions for the name contracts
  in `nickel/tests/`.

## Open

- Whether `'git` reads and writes refs through the `git` binary or a library
  (`gix`), and how it authenticates; whether `'forge` uses the host's API
  directly.
- Pull requests as a place of their own (a work package's PR), or only as
  a signal of drift (decision 6).
- Whether ADR 0005's `storage` axis is better written as a location too (a
  cold-storage copy is "another place this entry lives").
- Whether an outside consumer files its config as an issue or a pull
  request (ADR 0007, open); this capability's first consumer lives outside
  the repository.
