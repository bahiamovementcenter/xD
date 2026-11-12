# Johnny.Decimal experiments

Nov 12, 2026. Variations on the core in [jd-formal.md](jd-formal.md) that are
not Johnny.Decimal (JD). Each one names the section it changes and ends with a
verdict. Nothing here is planned.

## E1. The base as a parameter

The core never depends on base 10. Write the digit set as `D_b = {0, …, b−1}`
and JD is the case `b = 10`. Every definition in 1.1–1.5 holds for any `b`.
(A larger base was remembered as a ChatGPT suggestion; it isn't in the two
saved threads.)

| | `b = 10` (JD) | `b = 16` |
| --- | --- | --- |
| Areas | 10 | 16 |
| Categories per area | 10 | 16 |
| IDs per category | 100 | 256 |
| IDs per system | 10⁴ = 10,000 | 16⁴ = 65,536 (exactly `uint16`) |
| Standard zeros (2.1), if they scale with the base | `.00`–`.09` | `.00`–`.0F` |
| Area written as | `10-19` | `10-1F` |

**Costs of `b = 16` for names people see:**

- **Natural sort breaks.** Natural ("version") sort reads `1A` as `1` then
  `A`, so it sorts *before* `10`. Plain `sort` and `ls` (en_US.UTF-8 and C)
  get it right; `sort -V` and `ls -v` don't. Checked 2026-11-12:

  ```
  sort -V, ls -v    sort, ls
  1A Taxes          10 Admin
  10 Admin          11 Me
  11 Me             1A Taxes
  ```

  Graphical file managers (Finder, Dolphin, Nautilus) sort naturally by
  default, so they likely show the wrong order too. Not tested.

- **Ambiguous numbers.** `10.10` means different things in base 10 and base 16.
  Each system would need to declare its base, and jd only parses `[0-9]`.
- **It works against JD's design.** JD keeps the limit at 10 on purpose: if you
  run out, the category is too broad. Raising the limits postpones that
  signal; it doesn't replace it.
- **Harder to say and remember.** "One-one dot one-three" is easier than
  "one-A dot three-F".

**Other bases.** Capacity is `b⁴` per system. Sorting was tested on every
`AC.ID` name in numeric order (bases ≤ 16), or every category (larger bases),
on 2026-11-12:

| Base | Alphabet | IDs/category | IDs/system | Bits | `sort`, `ls` | `sort -V` | Safe on case-insensitive disks |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 8 | `0-7` | 64 | 4,096 | 12.0 | ok | ok | yes |
| **10 (JD)** | `0-9` | 100 | 10,000 | 13.3 | ok | ok | yes |
| 12 | `0-9AB` | 144 | 20,736 | 14.3 | ok | wrong | yes |
| 16 | `0-9A-F` | 256 | 65,536 | 16.0 | ok | wrong | yes |
| 32 | Crockford (`0-9A-Z` minus `I L O U`) | 1,024 | 1,048,576 | 20.0 | ok | wrong | yes |
| 36 | `0-9A-Z` | 1,296 | 1,679,616 | 20.7 | ok | wrong | yes |
| 58 | Bitcoin base58 (mixed case) | 3,364 | 11,316,496 | 23.4 | wrong | wrong | no |
| 62 | `0-9A-Za-z` | 3,844 | 14,776,336 | 23.8 | wrong | wrong | no |

- **Above 10, natural sort breaks.** Any letter digit sorts before the
  two-digit numbers.
- **Above 36, everything breaks.** Mixed case needs case-sensitive sorting, and
  on macOS and Windows disks `1a` and `1A` are the same name. Base64 is out
  entirely: its `/` can't appear in a file name.
- **Below 10, too small.** Base 8 gives 64 IDs per category.
- **For machine keys,** Crockford base32 is the best of the larger bases. It
  leaves out `I L O U` so they can't be misread, and it's case-insensitive.

**Verdict:** keep `b = 10` for names. Use `b` as a parameter only so the formal
core stays general. Use hex for machines (E2) or inside one expanded area (E3).

## E2. Machine encodings (a 1.5 projection)

A projection only has to round-trip the number. So one ID can have encodings
for people and for machines at the same time, and only the decimal one is ever
shown to people.

- **Integer:** `AC.ID ↦ 100·(10a + k) + m`, from 0 to 9,999. That fits in
  14 bits, since 10⁴ < 2¹⁴ = 16,384. With a system prefix
  (`[A-Z][0-9]{2}`, 2,600 values, 12 bits), `SYS.AC.ID` fits in 26 bits.
- **Hex:** the same integer written in hex, e.g. `11.13` ↦ 1113 ↦ `0x0459`. Use
  it for storage keys and wire formats, never for folder names.
- **Spatial:** split the number into coordinates to place it, e.g.
  `(area, category, id) ↦ (x, y, z)`. Morton or Hilbert codes then interleave
  those coordinates back into one sortable key, so neighboring voxels are stored
  together (in chunks or a sparse octree). Hilbert keeps more locality than
  Morton.
- **Color:** derive `L, C, H` (lightness, chroma, hue) from the hierarchy, not
  from the raw digits. Area ↦ hue, category ↦ a hue offset inside the area's
  band, ID ↦ lightness. Siblings then look alike and areas look distinct.

The integer keeps the whole hierarchical order (sorting the integers sorts the
names), but it's one-dimensional. Placing IDs in a 3D world needs the
coordinate split.

**Verdict:** worth doing; it costs nothing on the names people see. The
integer and hex forms are a few lines of code. Spatial and color are the
interesting experiments (see Open).

## E3. A hex area (inside 2.6, expand an area)

JD allows one expanded area whose format after the area number is free. A hex
scheme fits there without touching the rest of the system: `80-89` stays the
area, and its entries are `8x.HH` with `HH` in hex. The sorting and ambiguity
costs from E1 still apply, but only inside that area.

**Verdict:** allowed by JD, but only worth it if one area really needs more
than 100 IDs per category.

## E4. A content digest per entry (Merkle tree)

Extend the entry in 1.3 from `(n, label, meta)` to `(n, label, digest, meta)`.
The number stays the identity and the label stays the name; the digest is the
**version**.

- ID: `digest` = a Merkle hash of the ID folder's contents (file names and
  bytes, recursively, like a Git tree).
- Category, area, system: `digest = H(sorted (n, child digest) pairs)`. The
  system's root digest is a snapshot of the whole tree.
- **Labels left out of the hash:** renaming doesn't change a digest, which
  matches "the number is the identity". Putting them in (like Git does) makes
  the digest cover everything. To be decided.

Why it's useful: integrity checks; sync (compare root digests, then descend
only into the parts that differ); finding duplicate content; and the
`data_snapshot` in the world seed from the ChatGPT threads.

The same pattern in other systems: Git trees, IPFS/UnixFS directories,
Gordian Envelope digest trees. For anything already in git,
`git rev-parse HEAD:<path>` gives a folder's tree hash (of the last commit)
with no new code. The JD
root isn't in git, and some folders live on other storage backends, so this
system would need its own hasher.

A digest makes a poor label: it changes on every edit, and nobody can say it
out loud.

**Verdict:** worth building once sync between machines matters; it's the
cheapest way to tell what changed.

## E5. Shorthand notation

A **shorthand** is the shortest string that pins down one place in the tree.
Because a number spells its own ancestors (formal 3.1), a shorthand needs only
the last number, plus a `+` title if it names a child. Expanding it to a path
needs the JDex, but only for the labels.

| Shorthand | Expands to |
| --- | --- |
| `11` | `10-19 Life admin/11 Me` |
| `11.11` | `10-19 Life admin/11 Me/11.11 Passport` |
| `11.11+ Renew` | `10-19 Life admin/11 Me/11.11 Passport/+ Renew` |
| `P76.11.11` | `~/P76/10-19 Life admin/11 Me/11.11 Passport` |
| `W0003` | `W0000-9999 Work packages/W0003~11.11 Renew passport` |

- **Expand:** `shorthand → path` walks up the digits (`11.11 → 11 → 1`) and
  looks up each level's label in the JDex.
- **Collapse:** `path → shorthand` parses only the last structural folder name
  (`+ Title` also needs its parent), because that name already holds the whole
  number. Labels drop out, so collapsing is lossless on identity.
- `W0003~11.11` is the same place as `W0003`; the `~11.11` is a check that has
  to agree with the JDex.

**Open: `W0003~11.11+ Title`.** JD gives no reading for it. Two possible
rules:

1. **Reject it** (today's rule): work packages have no `+` children, and the
   name would look like a second `W0003` to jd.
2. **`+` binds to the nearest number on its left:** the work package links to
   the child `11.11+ Title`, not just to `11.11`. That would let a work package
   point at one specific child, but it extends JD.

A full precedence order for the separators (`/` loosest, then `~`, `+`, `.`,
`-` tightest) would settle cases like this one. So would explicit brackets
(E6), which is the simpler fix.

**Verdict:** expansion and collapse are worth building (they power `jd`
commands that take `11.11` and find the folder). The `~…+` extension needs a
real use case first.

## E6. Encapsulators

Bracket pairs that wrap any element from formal 3.1 (a number, a label, or a
whole shorthand) and say how to read it. Each pair reuses a meaning it already
has elsewhere:

| Pair | Meaning | Existing convention | Example |
| --- | --- | --- | --- |
| `( )` | grouping: read this part first | math, most grammars | `W0003~(11.11+ Title)` |
| `{ , }` | a set: one address per option | shell brace expansion | `11.{11,12,13}` = three IDs |
| `< >` | a placeholder for a value | man pages, this repo's docs (`<jj change ID>`) | `11.13+ <change ID>` |
| `[ ]` | an ordering hint inside a label | JD subfolder patterns (`Butcher, Lucy [03]`) | `Butcher, Lucy [03]` |

**Grouping settles E5's open case** without a precedence rule:

- `W0003~(11.11+ Title)`: the work package links to the child of `11.11`.
- `(W0003~11.11)+ Title`: a child of the work package. JD has no such thing,
  so it's rejected unless that changes.
- `W0003~11.11+ Title` with no brackets stays ambiguous, so it's an error that
  asks for brackets.

**Only in shorthand, never in folder names:**

- `< >` can't be used in Windows file names. `( )`, `[ ]` and `{ }` can, but
  the shell treats them specially (subshell, glob, brace expansion), so names
  with them need quoting.
- `[ ]` is already JD's, inside labels. A parser that stops at the first space
  (formal 1.5) never reads a label, so there's no clash.
- Expansion (E5) removes the brackets: `11.{11,12}` expands to two paths, and
  `( )` disappears once the grouping is applied.

**Verdict:** worth adding to the shorthand grammar. `( )` and `{ }` cost
nothing and fix real ambiguity; `< >` is for docs and templates only.

## E7. Role brackets (alternative to E6)

The same bracket pairs, but each one marks a **role** instead of a reading
order. Once every part is marked, spaces and the order of parts carry no
meaning. E6 and E7 give `[ ]` and `( )` different meanings, so a shorthand
grammar can adopt only one of them.

| Pair | Holds | Example |
| --- | --- | --- |
| `[ ]` | a number (the identity) | `[11.11]`, `[W0003]`, `[10-19]` |
| `( )` | a label, owned by the number just before it | `[11.11](Passport)`, `[10-19](Life admin)` |
| `{ }` | a link to another number | `[W0003]{[11.11]}` |
| `+` | a child; its title goes in `( )` | `[11.11]+(Wendy)` |

| Plain shorthand (E5) | Role brackets | Meaning |
| --- | --- | --- |
| `11.11` | `[11.11]` | ID `11.11` |
| `10-19 Areas` | `[10-19](Areas)` | area `10-19`, labeled Areas |
| `W0003~11.11 Title` | `[W0003](Title){[11.11]}` | work package linked to `11.11` |
| none | `[W0003](Title){[11.11]+(Wendy)}` | work package linked to the child Wendy of `11.11` |
| `W0003~11.11+ Title`, one reading | `[W0003]{[11.11]}+(Title)` | a child of the work package; JD has none, so rejected |

**Gains over E6:**

- A label ends only at its closing `)`, so it can hold spaces, `.`, `+`, `~`
  and JD's own `[03]` (`(Butcher, Lucy [03])`). Brackets must balance.
- Every case from E5 has exactly one form, so there's no ambiguity left and no
  precedence to decide.

**Costs:**

- More characters than plain shorthand for the common case (`[11.11]`
  vs `11.11`).
- It's further from how JD writes things. Like E6, it's for shorthand only;
  folder names stay plain JD.

**Verdict:** E7 is the stricter, machine-friendly form; E6 is lighter for
typing. A tool could accept plain E5 shorthand, use E6 to disambiguate, and
print E7 when it must be exact. Undecided.

## E8. Dot entries: `11..<convention>`

A hidden file or folder inside any node, written as the node's number, the
level separator `.`, then the hidden name with its own leading `.`:

| Shorthand | Path | Owned by |
| --- | --- | --- |
| `11..jd.ncl` | `11 Me/.jd.ncl` | this repo's distributed config |
| `11.12..git` | `11 Me/11.12 Website/.git` | git, for `'repo` storage |
| `10-19..stfolder` | `10-19 Life admin/.stfolder` | Syncthing |
| `00.00..obsidian` | `00-09 …/00 …/00.00 JDex/.obsidian` | a note app |
| `11..owners` | `11 Me/.owners` | ownership (E11) |

This makes them a **third kind of child**, alongside IDs and `+` children:

| Child | Numbered | Shown in a file manager | Owned by |
| --- | --- | --- | --- |
| ID (`11.11`) | yes | yes | the person |
| `+` child (`11.11+ Wendy`) | no | yes | the person |
| dot entry (`11.11..git`) | no | no | a tool or convention |

- **Parsing is unambiguous.** After a number, `.` is followed either by digits
  (the next level) or by another `.` (a dot entry). A number never starts
  with `.`.
- **jd already skips them.** A dot name isn't `11.11` or `11.11 *`, so
  formal 1.5's `parse` maps it to `skip`. This repo's `stray_dotfile` warning
  is a rule about one of them (`.jd.ncl`) appearing in the wrong place.
- **Formally:** a set of entries keyed by `(n, name)`, where `name` starts
  with `.`, outside the number space, and never in the JDex. A registry of
  known conventions (`.jd.ncl`, `.git`, `.stfolder`, …) can say which tool owns
  each one and which levels it may appear at.

**Clashes to watch:**

- `..` means "parent folder" in paths. A dot entry is shorthand, never a path,
  so the two only meet if shorthand is typed where a path is expected.
- `..` is a range in Rust, Ruby and bash (`{11..19}`). If E6's `{ }` sets ever
  allow ranges, `11.{11..19}` would mean IDs `11.11`–`11.19`, not a dot entry.
  The `{ }` context keeps it distinguishable, but E6 should pick a different
  range syntax (e.g. `11.{11-19}`, reusing `-`) to keep `..` single-purpose.

**Verdict:** worth adding to the shorthand grammar. It names things the design
already has, costs no number space, and gives tool-owned metadata a place next
to the node it describes.

## E9. `.05` as a hex-numbered slot (mandatory `+` children)

Every `.05` (the AI standard zero, formal 2.1) holds nothing directly; all its
content lives in `+` children titled with a two-digit hex number:

```
11 Me/
└── 11.05 AI/
    ├── + 00 Index/        reserved, mirrors .00
    ├── + 01 Claude/
    ├── + 02 Local model/
    └── + 0A …/            up to + FF
```

Shorthand: `11.05+ 01`. JDex entries: `11.05+ 01 Claude`.

- **JD allows it.** Extend-the-end is meant for "specific categories or IDs",
  and one standard zero is that. JD also allows keeping everything in `+`
  subfolders.
- **255 slots, one byte.** `01`–`FF` fits a `uint8` (E2), with `00` kept as
  the slot's own index, like `.00` for a category.
- **The rule is a predicate on `.05`:** no direct files, every child matches
  `+ [0-9A-F]{2}( label)?`, titles unique (JD's existing rule).
- **The hex is a title, not a number.** JD's `+` children take no number, so
  this is a naming convention inside one ID and doesn't touch formal 1.1 or 1.4.

**Costs:**

- **JD's own tools claim `.05`.** The standard zeros page says JD's skills and
  MCP server "assume they have control over AC.05". A mandatory layout here
  can collide with what they write. Check what they create before adopting
  this.
- **Natural sort is wrong.** Tested 2026-11-12 on `+ 01`–`+ FF`: plain `sort`
  and `ls` are correct; `sort -V` gives `… + 3D, + 3E, + 3F, + 04, + 4A …`.
  Graphical file managers likely show the same.
- **A bare hex title says nothing.** Use `+ 0A Label`, not just `+ 0A`.

### Proposed layout: slots and columns in one byte

The low hex digit splits the range by meaning: `1`–`9` are fixed kinds of AI
configuration (the same in every `.05`, like standard zeros), and `A`–`F` are
six workflow columns.

```
11.05 AI/
├── + 00 Index/
├── + 01 Agents/             kinds of configuration
├── + 02 Skills/
├── + 03 Rules/
├── + 04 …/                  up to + 09
├── + 0A Inbox/              columns
├── + 0B Anti-library/
├── + 0C Backlog/
├── + 0D To do/
├── + 0E Doing/
└── + 0F Done/
```

- **The high digit can name a board.** `0A`–`0F` is board 0. `1A`–`1F` could
  be a second board (say, a publishing workflow: Draft, Edit, Review, …).
  Whether `11`–`19` repeat the kinds or mean something else is open.
- **Sort order, tested 2026-11-12 on `+ 00`–`+ 0F`:** plain `sort` and `ls`
  keep hex order. `sort -V` puts `0A`–`0F` *before* `01`–`09`, so a file
  manager would likely show the board above the slots. From `1A` on, natural
  sort interleaves boards (see above).
- **Labels go after a space, not in `< >`:** `+ 0A Inbox`. `< >` can't be used
  in Windows file names and means "placeholder" in E6.

**Moving a card is a folder move.** Moving a card from `+ 0C Backlog` to
`+ 0E Doing` changes its path, which strands jd's journaled moves
(`strands_moves`) and breaks links by path. Two ways around it:

1. **Cards keep a stable identity:** each card is a work package or a `+`
   child with its own name, and the column folder holds only a link to it.
2. **Columns are a status, not folders:** the Org JDex already has TODO
   keywords (`TODO`, `DOING`, `DONE`). The Kanban board becomes a view of the
   JDex, and `0A`–`0F` become the six keyword values instead of folders.

**The kinds `01`–`09`.** Fixed, so `11.05+ 02` always means "skills for
category 11". A candidate list, taken from what agent tools already separate
(Claude Code's `.claude/`: `agents/`, `skills/`, `commands/`, hooks, settings,
memory; MCP server configs; `AGENTS.md`/`CLAUDE.md` rules files):

| Slot | Kind | Agrees with |
| --- | --- | --- |
| `01` | Agents | `.claude/agents/` |
| `02` | Skills | `.claude/skills/` |
| `03` | Rules | `CLAUDE.md`, `AGENTS.md` |
| `04` | Commands | `.claude/commands/` |
| `05` | Hooks | Claude Code hooks |
| `06` | MCP servers | `.mcp.json` |
| `07` | Memory | Claude Code memory |
| `08`–`09` | free | |

Tools look for these at fixed dot paths, not in `+ 02 Skills`. So the `+`
folders are where people keep them, and E8 dot entries link to them: for
example `11..claude/skills` → `11.05+ 02 Skills`.

**Overlap with JD:** `0A Inbox` duplicates the category's own `.01 Inbox`, and
`0F Done` overlaps `.09 Archive`. Either say how they differ (this inbox
is the agent's, `.01` is yours) or drop them from the board.

**Open:** folders (1) or status (2)? The final list of kinds `01`–`09`.
**Verdict:** the byte layout is neat. Settle JD's claim on `.05` first, then
the folders-or-status question.

## E10. Identity, label, pointer, proof

Formal 1.3 has two slots, `n` (identity) and `label`. Splitting them by the
question each answers gives four, and each can hold more than a JD number or a
name:

| Slot | Answers | JD today | Could also hold |
| --- | --- | --- | --- |
| Identity | which thing? | `P76.11.11` | UUID, ULID, DID, DOI, URN, envelope digest |
| Label | what do people call it? | `Passport` | other languages, an icon, a color (E2), a handle |
| Pointer | where is it? | the folder path | URL, `ipfs://`, `s3://`, git remote, symlink |
| Proof | is it the right content? | none | content digest (E4), signature |

### Identity is a chain of namespaces

Each level is unique only within the one above it:

```
owner (DID or domain) → system (P76) → AC.ID (11.11) → + child / .. entry
globally unique         unique per owner  unique per system  unique per ID
```

- **The system code is the namespace JD already has** (formal 2.2), but
  `A00`–`Z99` is only unique per person: two people's `P76` collide.
- **The owner level is new.** It's needed once systems are shared or
  published. Options: a DID (`did:key:z6Mk…`, self-certifying, no registry, and
  the same key can sign envelopes, so identity and proof share a root), a
  domain (DNS), or an informal URN (`urn:jd:<owner>:P76:11.11`; `jd` isn't a
  registered URN namespace).
- **On disk, identity is split.** The root folder (`~/P76`) supplies the
  owner and system; each folder name carries only `AC.ID`.

### Each slot holds a set, with one primary

```
identities: { jd: P76.11.11 (primary), uuid: 0193…, did: did:key:z6Mk…/P76.11.11 }
labels:     { en: Passport (primary), pt: Passaporte, icon: 🛂 }
pointers:   [ file:///~/P76/…/11.11 Passport, s3://bucket/…, ipfs://bafy… ]
proofs:     { digest: sha256:… }
```

This is a Gordian Envelope: identity as the subject, the rest as assertions,
each one separately elidable or encryptable.

### What can be the primary identity

It must keep what formal 1.3–1.5 rely on:

| Scheme | Unique | Stable | Short | Useful sort | Holds its parent | Role |
| --- | --- | --- | --- | --- | --- | --- |
| JD number | yes | yes | yes | yes | yes | primary |
| ULID | yes | yes | 26 chars | by creation time | no | flat spaces (like work packages) |
| UUID v4 | yes | yes | 36 chars | no | no | alias |
| DID, DOI | yes | yes | long | no | no | alias; the owner level |
| Content hash | yes | **no** (changes on edit) | long | no | no | proof only (E4) |

### What can be the primary label

On disk it must be a valid file name: no `/`, none of Windows' `< > : " \ | ? *`.
So a URL can't be a label. A pointer goes in the pointer slot, and the label
can be a short name that resolves to it. The design already does this once:
`'path` storage makes the folder a symlink, a pointer where contents would be.

### The rule

- **Folder name** = primary local identity + space + primary label
  (`11.11 Passport`). Nothing else.
- **Root folder** = the system (and, once shared, the owner).
- **Everything else** (aliases, other labels, pointers, proofs) lives in the
  metadata layer: the JDex, `.jd.ncl`, or a Gordian Envelope. That layer is
  where the shared network and the generated world from the ChatGPT threads
  attach.

**Verdict:** the four-slot split is worth adopting in the formal model now; it
costs nothing on disk. The owner level waits until a system is shared.

## E11. Owners as files in the tree

Who owns a node is not part of any name or shorthand (an earlier `@actor`
notation was dropped). It lives in a hidden `.owners` file inside the node's
own folder, an E8 dot entry (shorthand `11..owners`), so ownership follows the
hierarchy and stays out of file-manager view:

```
~/P76/
├── .owners                    P76..owners: system owners
├── 10-19 Life admin/
│   └── 11 Me/
│       ├── .owners            11..owners: category 11 and everything in it
│       └── 11.11 Passport/
│           └── .owners        11.11..owners: overrides for 11.11 only
```

- **Nearest file wins.** A node without a `.owners` takes its nearest
  ancestor's, the way most-specific rules win in CODEOWNERS. A file can opt
  out of inheriting (Chromium's `set noparent`).
- **Prior art.** Per-directory owner files are how Chromium (`OWNERS`),
  Kubernetes (`OWNERS`, with separate `approvers` and `reviewers`) and
  Gerrit's code-owners plugin work. GitHub, GitLab and Forgejo read only one
  `CODEOWNERS` per repo, so jd compiles the `.owners` files into that one,
  as workflow-rules.md already does for per-area `owners`.
- **Owns vs. may act.** Kubernetes' split answers it: `approvers` approve
  changes, `reviewers` (or an agent's working scope) may act. One file can hold
  both.
- **Actors are identities** (E10): each entry resolves to a DID or a host
  handle; users, agents and groups only differ in what that identity says.
- **Policy** attaches as a `+` child of the owners file's node or as a
  section in the file, compiled together with it into the host's review and
  `protect` rules.

**Which repo holds them.** In the distributed layout the root is a jj repo
that tracks `.jd.ncl` files and the JDex, so `.owners` files sit next to
them. In the central layout the folders aren't in a repo, so ownership stays
the `owners` field in the config; a `.owners` file on disk would be a disk
edit for `jd plan` to record.

**Decided (2026-11-12):** the file is `.owners`, an E8 dot entry. Only the
compiled `CODEOWNERS` is read by the git host.

**Verdict:** adopt; it extends what shared systems already need and keeps
names free of ownership.

## Open

1. E2 color: test whether people can see the groupings. Render one real JD
   tree as swatches and check if areas and categories are recognizable without
   labels.
2. E2 spatial: choose the coordinate split. `(area, category, id)` gives a
   10×10×100 box; something else may tile better.
