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

## Open

1. E2 color: test whether people can see the groupings. Render one real JD
   tree as swatches and check if areas and categories are recognizable without
   labels.
2. E2 spatial: choose the coordinate split. `(area, category, id)` gives a
   10×10×100 box; something else may tile better.
