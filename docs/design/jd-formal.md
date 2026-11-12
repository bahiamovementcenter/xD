# Johnny.Decimal, formally

Nov 12, 2026. Draft. Goal: pin down Johnny.Decimal (JD) as a small **core** plus
**independent modules**, so each module has a clear interface and can be swapped
for, or composed with, an existing conventional system.

Notation: `D = {0,…,9}`. A two-digit number `xy` is the pair `(x, y) ∈ D²`.
"This repo" means a choice made in [workflow-rules.md](workflow-rules.md), not
the JD standard.

---

## 0. Reading guide

**The form of the notation says what something is.** The shape of a number
gives its kind, and the symbol beside it gives its relation to something else.

**Shapes: what kind of thing**

| Thing | Form | Why |
| --- | --- | --- |
| Area | range, `10-19` | a span of categories, written as its first and last |
| Category | integer, `11` | one point in that span; its first digit is its area |
| ID | decimal, `11.11` | a position inside a category: category, `.` "within", slot |
| System | letter + 2 digits, `P76` | the letter means outside the decimal tree: a namespace (2.2) |
| Work package | letter + 4 digits, `W0011` | outside the tree too, and flat (2.4) |
| Expanded area | area digit + anything, `20.JOE.BB` | keeps the area digit, then its own scheme (2.6) |

**Zero: the level's own management**, at every level (2.1): system `00-09` /
`00` / `00.00`, area `x0` (e.g. `10`), category `.00`–`.09`, work packages
`W0000`–`W0010`.

**Symbols: how it relates** (full table in 3.1)

| Symbol | Relation | Example |
| --- | --- | --- |
| `-` | from … to | `10-19` |
| `.` | inside, one level down | `11.11` |
| `+` | a child that takes no number | `11.11+ Wendy` |
| `~` | a link to an ID in the other tree | `W0011~11.11` |
| ` ` | end of the number, start of the label | `11.11 Passport` |
| `/` | one level down, on disk | `10-19 …/11 …` |

Four rules:

- Before the first space is identity; after it is a label you can change.
- A number holds its own path: `11.11` contains `11` and area `1`.
- Digits mean inside the decimal tree; a leading letter means outside it.
- Anything that takes no number is marked by a symbol (`+`) instead.

Experiments add `..` for hidden, tool-owned entries and a hex `.05` slot
(jd-experiments.md, E8 and E9). They aren't JD.

---

## 1. Core

### 1.1 Address space

| Level | Set | Written | Example |
| --- | --- | --- | --- |
| Area | `a ∈ D` | `a0-a9` | `10-19` |
| Category | `(a, k) ∈ D²` | `ak` | `11` |
| ID | `(a, k, m) ∈ D² × D²` | `ak.mm` | `11.11` |

The level is the tuple length, so `|Areas| = 10`, `|Categories| = 100`
(10 per area) and `|IDs| = 10⁴` (100 per category).

### 1.2 Hierarchy

`parent` drops the last component: `parent(a,k,m) = (a,k)`, `parent(a,k) = (a)`.
The tree is the **prefix order** on tuples. The parent of a number is
**computed from its digits**, so a category can't move to another area and an
ID can't move to another category. Moving one means giving it a new number
(the repo error `category_outside_area`).

### 1.3 Entries

An entry is `e = (n, label, meta)`, where `n` is an address from 1.1.

- **Identity:** two entries are the same thing iff their `n` is equal. `label`
  is a mutable display string, so changing it is a rename.
- **Uniqueness:** within one system, `n ↦ e` is a partial function. At most one
  live entry has each number (the repo error `duplicate_number`).
- **Closure:** if `e` exists, an entry for `parent(n)` exists too.

### 1.4 Allocation and retirement

Each system keeps two sets: `Live` and `Retired`, with `Live ∩ Retired = ∅`.

- **Allocate:** `next(c) ∈ {m : (c,m) ∉ Live ∪ Retired}`, also excluding the
  numbers that 2.1 reserves.
- **Never reuse:** `Retired` only grows. Archiving or renumbering moves `n`
  from `Live` to `Retired`. JD (the-jdex): "I never re-use IDs"; the JDex is
  the master record of every ID assigned, which is what makes this checkable.
- The *choice* inside `next` is a policy parameter. The JD docs only say IDs
  are "ordered in the sequence that you created them" (so in practice
  `max + 1`), and work packages "increment by 1". The invariant is the only
  hard requirement.
- **Capacity is fixed:** the JD docs give no IDs above `.99`. Running out
  means the category is too broad and should be split.

### 1.5 Projections (renderings)

A projection is a pair `render: Entry → String` and
`parse: String → Address ∪ {skip}` with `parse(render(e)) = e.n`. It must be
lossless on the number and is allowed to drop the label.

| Projection | `render` | `parse` accepts |
| --- | --- | --- |
| Folder name | `11.11 Passport` | `11.11`, `11.11 *` |
| JDex note / Org heading | heading text, `CUSTOM_ID = 11.11` | `CUSTOM_ID` |
| Area folder | `10-19 Life admin` | `10-19`, `10-19 *` |

Anything `parse` maps to `skip` is outside the system. That rule is what
makes the extensions in section 2 possible without collisions.

---

## 2. Modules (each optional; dependencies noted where they exist)

### 2.1 Standard zeros: reserved slots

Reserves a fixed sub-range at every level for managing that level. The same
ten slots repeat at three scopes
([JD docs](https://johnnydecimal.com/documentation/the-standard-zeros)):

| Scope | Where | Example |
| --- | --- | --- |
| Category | `ak.00`–`ak.09` | `11.09` archive of category `11` |
| Area | category `a0`, IDs `a0.00`–`a0.09` | `10.09` archive of area `10-19` |
| System | area `00-09`, category `00`, IDs `00.00`–`00.09` | `00.00` the system's JDex, "the very first ID" |

| Slot | Meaning |
| --- | --- |
| `.00` | JDex |
| `.01` | Inbox (must be tidied regularly) |
| `.02` | Task and project management |
| `.03` | Templates |
| `.04` | Links |
| `.05` | AI: a place for an AI agent (added 2026-09-07) |
| `.06`–`.08` | Reserved for expansion, do not use |
| `.09` | Archive |

- When saving, prefer the most specific zero (category before area before
  system).
- **First regular ID: `.11`** (JD docs). They don't say what `.10` is; this
  doc treats it as reserved, so regular IDs are `.11`–`.99`.

Formally: a predicate `reserved(n)` that `next` excludes, and a `scope(n)`
function mapping `.0x` slots to category, area or system. Usable capacity
becomes 9 areas × 9 categories × 89 IDs.

### 2.2 Multi-system prefix

`SYS = [A-Z][0-9]{2}`, so `A00`–`Z99` is 2,600 systems (JD docs; this repo:
`P76`, `D25`). The full address is `SYS.ak.mm`, e.g. `H01.11.11`. The docs
say to use it only when systems share a domain. Each `SYS` is its own space
with its own copy of 1.3 and 1.4. The prefix only selects which space; there is no relation between `P76.11.11` and
`D25.11.11`.

### 2.3 Extend-the-end (`+` children)

`ak.mm+ Title` (JD writes it `AC.ID+`; examples are `11.63+ Wendy` and
`72.02+ X01`, always with a space, though no rule requires one) adds a child
entry under an ID. The JD docs make each one its own JDex entry, with a unique
title, sorted below its parent. On disk it is a subfolder **inside** the ID
folder, named `+ Title` (`11.63 Veterinary care/+ Wendy/`), sorting above the
ID's other subfolders. It **does not use up a number**: `parse` maps
it to `skip` for allocation and to `ak.mm` for attachment. Formally: a set of
child entries keyed by `(ak.mm, title)`, one level deep, outside the number
space. JD limits it to specific categories or IDs and forbids it together with
expand-an-area.

### 2.4 Work packages

A **second, flat number space** `W0000-9999`, which belongs to the system,
sits outside any area and has no categories
([JD docs](https://johnnydecimal.com/documentation/work-packages)).

- It allocates under 1.4: start at `W0011`, increment by 1. `W0000`–`W0010`
  are its standard zeros; old work goes in `W0009~00.09 Archive`.
- A **foreign key**: each work package names exactly one ID with a tilde
  (`W0011~11.13 Compliance audit`). JD says "This is required"; this repo
  enforces it as `wp_without_id`.
- No `+` children; inside, ordinary subfolders in blocks of ten
  (`05 Planning` … `95 …`).

### 2.5 Archive strategy (this repo)

A function `archive_target: Address → Location` with four values
(`standard`, `last_slot`, `in_place`, `separate_root`; see workflow-rules.md).
It only decides **where** retired entries go. The fact of retirement is
already fixed by 1.4. JD itself has only `standard`: archive into the nearest
`.09` (needs 2.1). The other three are this repo's additions.

### 2.6 Expand an area

One area (and only one, "only expand the part that needs to be expanded")
drops the 1.1 capacity limits and "can grow to an essentially infinite size".
Its only rule: every entry still **starts with the area number** (`83.66`
belongs to `80-89`). After that prefix the format is free: numbers in tens
(`10, 20, 30`, leaving room for `05`, `15`), dates as `yyyy-mm-dd`, or invented
codes (`20.JOE.BB.20240708`). Works with 2.2; forbidden with 2.3.

Formally: for one area `a`, replace the `ak.mm` level with `a · Suffix`, where
`Suffix` is any string whose parse is defined by that area's own scheme. This
is the hook where a different identification scheme plugs into JD.

### 2.7 Subfolder patterns

Below an ID, JD allows exactly one more level of folders, and they must follow
a pattern, never ad hoc names: dates (`yyyy-mm-dd`, `yyyy-mm`, `yyyy-q1`,
`yyyy-ww`), consistent alphabetical names (optionally with `[03]`), or the
fallback `10` … `90` (nine folders, `15` squeezed between if needed).

Formally: an ID folder's children are a set of `+` children (2.3) plus at
most one pattern `P ∈ {date, alpha, tens}` that all other subfolders match.

---

## 3. Grammar (ABNF, RFC 5234)

```abnf
address      = [ sys "." ] ( area / category / id )
sys          = %x41-5A 2DIGIT              ; A-Z only (ABNF strings ignore case)
area         = DIGIT "0-" DIGIT "9"        ; same leading digit, checked separately
category     = 2DIGIT
id           = category "." 2DIGIT
child-entry  = id "+" SP label             ; 2.3, JDex entry title, as the docs write it
child-folder = "+" SP label                ; 2.3, folder inside the ID folder
wp           = "W" 4DIGIT                  ; 2.4
wp-entry     = wp "~" id [ SP label ]
entry-name   = ( address / wp ) [ SP label ]
label        = 1*( VCHAR / SP )
```

JD's own placeholders (acid-notation): `AC.ID` any ID, `1C.ID` any ID in
area `10-19`, `11.ID` any ID in category `11`, `AC.11` ID `.11` in any
category, `SYS.AC.ID` with a system prefix.

Not covered: entries of an expanded area (2.6), whose format is per area.

Semantic constraints that ABNF can't express: an area's two digits match
(`1` and `1` in `10-19`); a category's first digit is its area; 2.1's
`reserved` predicate; the uniqueness and retirement rules of 1.3 and 1.4.

### 3.1 Separators

Six characters carry all the structure. Everything after the first space is a
label, so a label may contain any of them.

| Separator | Between | Meaning | Example |
| --- | --- | --- | --- |
| `-` | first and last category of an area | a range | `10-19` |
| `.` | system and category; category and ID | the next level **inside one number** | `P76.11.11` |
| ` ` (space) | a number and its label; `+` and its title | end of the number, start of the label | `11.11 Passport`, `+ Wendy` |
| `+` | after an ID (JDex), or at the start of a folder name | a child that takes no number | `11.63+ Wendy`, `+ Wendy/` |
| `~` | a work package and its ID | a link to an ID in the other tree | `W0011~11.13` |
| `/` | folders | the next level **on disk** | `10-19 Life admin/11 Me` |

`.` and `/` both mean "one level down". A number spells its own ancestors
(`11.11` contains `11` and area `1`), so a path repeats digits that the number
already holds: `10-19 …/11 …/11.11 …`. That redundancy is what makes a short
form possible (experiments, E5).

---

## 4. Seams: where an existing system could plug in

Each row is one module's interface and the established conventions that
already solve the same problem. These are what to evaluate next, one row at a
time.

| Module | What it fixes | Conventional systems with the same shape |
| --- | --- | --- |
| 1.1–1.2 address + prefix tree | fixed-depth decimal hierarchy | Dewey Decimal / UDC (decimal classification); CIDR (`10-19` is the prefix `1*`) |
| 1.3 identity ≠ label | number is identity | DOI and handles (persistent ID, mutable metadata); content addressing (CID) vs names |
| 1.4 never reuse | append-only registry | IANA registries, DOI, database sequences |
| 1.5 projections | lossless number round-trip | serialization formats; Org `CUSTOM_ID`; URI templates |
| 2.1 reserved slots | management sub-ranges | reserved IP ranges (RFC 1918), well-known ports |
| 2.2 system prefix | namespace selection | URN namespace IDs (RFC 8141), DNS labels |
| 2.3 `+` children | attach without allocating | URI fragments (`#`), DOI suffixes |
| 2.4 work packages | flat ID with a foreign key | ticket keys (`PROJ-123`), Git refs to commits |
| 2.5 archive strategy | where retired things live | storage tiering; tombstones in CRDTs and DNS |
| 2.6 expand an area | a foreign scheme under one area prefix | any of the others: dates (ISO 8601), UUIDs, DOIs, ticket keys |
| 2.7 subfolder patterns | one fixed pattern per ID | ISO 8601 dates; collation rules |

## Open questions

1. `.10`: the JD docs list only `.00`–`.09` as reserved and start IDs at
   `.11`. Treat `.10` as reserved (as above) or as free?
2. Depth: JD allows exactly one more level (2.7) plus extend-the-end, with
   2.6 as the escape hatch. Dewey and UDC go arbitrarily deep. Is 2.6 enough,
   or should depth be a parameter of the core?
3. The JDex field format is specified separately at
   github.com/johnnydecimal/index-spec (not read yet).

Sources checked 2026-11-12: johnnydecimal.com pages the-standard-zeros, ids,
areas-and-categories, multiple-systems-overview/guidelines,
extend-the-end-overview/guidelines, work-packages, inbox-archive,
acid-notation, the-jdex, jdex-other-methods, expand-an-area-overview/guidelines,
subfolder-patterns. Not readable: /sbs/ pages (sign-in only).
