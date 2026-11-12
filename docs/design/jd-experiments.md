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

## Open

1. E2 color: test whether people can see the groupings. Render one real JD
   tree as swatches and check if areas and categories are recognizable without
   labels.
2. E2 spatial: choose the coordinate split. `(area, category, id)` gives a
   10×10×100 box; something else may tile better.
