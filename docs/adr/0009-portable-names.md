# 9. Names that work on every file system xD writes to

Status: proposed, 2026-11-14.

## Context

xD writes folder names (`11.11 Passport`), and once placement (ADR 0005) and
apply (ADR 0002) exist, whole paths. Ash asked (2026-11-14) how xD tests that
those names work on real file systems. Today only the number part is checked:

- `Scheme::portability()` warns when an alphabet's digits differ only in case
  (`a` and `A` in base 62), and when a digit or separator is one of
  `< > : " / \ | ? *`.
- `AlphabetIsUsable` in `nickel/xd.ncl` rejects `/` and whitespace in digits.
- `crates/xd-core/tests/folders.rs` writes real folders for every base from
  2 to 64, and CI runs it on macOS, whose disks ignore case.

Nothing checks the **label**, so `11.11 Pass/port`, `11.11 CON`,
`11.11 Taxes.` and `11.11 Taxes ` (a trailing space) are all accepted. Nothing
checks Windows' reserved names, Unicode normalization or length limits, and
CI doesn't run on Windows.

There is no single official specification of a portable file name. Each
rule has its own authority:

| Source | What it says | Used for |
| --- | --- | --- |
| POSIX.1-2024, Base Definitions, "Filename" | A name may hold any byte except `/` and NUL | `posix` |
| POSIX.1-2024, Base Definitions, "Portable Filename Character Set" | Only `A–Z a–z 0–9 . _ -` are guaranteed everywhere | Not adopted (decision 2) |
| POSIX `NAME_MAX`; Linux `limits.h` | 255 bytes per name on Linux file systems | `posix` |
| Microsoft, "Naming Files, Paths, and Namespaces" | Forbidden characters `< > : " / \ \| ? *` and codes 0–31; reserved names `CON PRN AUX NUL COM0–COM9 LPT0–LPT9` (and `COM¹ COM² COM³ LPT¹ LPT² LPT³`), also when followed by an extension (`NUL.txt`); no trailing space or period; `MAX_PATH` of 260 characters unless long paths are enabled; names usually at most 255 characters | `windows` |
| Apple, "Apple File System Guide" and APFS reference | APFS volumes on macOS are case-insensitive by default; APFS keeps the normalization a name was written in but compares names without regard to it; HFS+ stored names decomposed; 255 UTF-8 bytes per name | `macos` |
| Unicode Standard Annex #15, "Unicode Normalization Forms" | NFC and NFD, and that canonically equivalent strings should be treated as equal | all |
| Unicode `CaseFolding.txt` | Default case folding, for comparing names case-insensitively | `macos`, `windows` |

ADR 0008 (pull request #10, not yet merged) already plans to keep rules like
these as reviewed data, one entry per target with its source and the date it
was checked, in `nickel/targets.ncl`, and names `posix`, `macos` and
`windows` among its targets. This ADR fills in those three entries and says
where xD checks them.

## Decisions

### 1. One Nickel contract per file system, each citing its source

`nickel/targets.ncl` holds an entry per file system, with the fields ADR 0008
lists (allowed characters, case behaviour, length limits, reserved names,
`source`, `checked`). Each entry gets a small contract built from its data:

- **`posix`**: no `/`, no NUL, not `.` or `..`, at most 255 UTF-8 bytes.
- **`windows`**: no `< > : " / \ | ? *`, no code points 0–31; not a reserved
  name, compared case-insensitively and with any extension removed (`con`,
  `NUL.txt`, `Com1.tar.gz`); no trailing space or period; at most 255 UTF-16
  code units. Case is ignored when comparing names.
- **`macos`**: no `/` and no NUL, at most 255 UTF-8 bytes. Case and
  normalization are ignored when comparing names. (`:` is allowed by APFS but
  shown as `/` in Finder; the `windows` rule already excludes it.)

A rule that turns out wrong is fixed in the data, with its source, and the
contracts follow.

### 2. `portable` is the default, and the only one a configuration sees

**`portable`** is `std.contract.all_of [posix, windows, macos]`, plus one rule
of its own: names xD writes are in **NFC**. Every name xD writes must satisfy
it. Its effective rules are the strictest of each: Windows' characters,
reserved names and trailing rule; 255 UTF-8 bytes (which also keeps a name
under 255 UTF-16 units); case-insensitive, normalization-insensitive
comparison.

The POSIX portable character set is not adopted: it excludes spaces, and a
Johnny.Decimal name has a space before its label. xD keeps to what the three
file systems actually accept.

`posix`, `windows` and `macos` exist only as the parts `portable` is built
from, and so CI can check each against its own operating system (decision 6).
Letting a configuration pick a looser target (a Linux-only tree whose labels
contain `:`) is a mechanism with no user yet, so per extensibility.md it isn't
built: it is registered as the deferred capability `file_system_targets`, and
built once two configs need it.

### 3. Labels are checked where they enter xD

- **Nickel**: the label a configuration gives (the `label` argument of a
  pattern's `name`, and entries' labels once entries exist) is checked with
  `portable` at evaluation, so a bad label fails with the rule it breaks and
  its source, such as `"CON" is reserved on Windows (Microsoft, Naming Files,
  Paths, and Namespaces)`.
- **Rust core**: `name::render` already returns a `Result`; it returns an
  error for a label `portable` rejects. The agreement test between Nickel and
  Rust covers it.
- **The number part** stays with `Scheme::portability()`, which keeps its
  warnings but takes its character list and case rule from the same data
  instead of its own `UNPORTABLE` constant. A rendered number that ends in
  `.` or a space (a separator at the end) is reported too.

A label that `portable` rejects is fixed by renaming it. The label isn't
identity (formal 1.3), so this never touches a number.

### 4. Names read from disk are reported, not rejected

Folders that already exist may break these rules (a label with `:` written on
Linux). Reading them must not fail: `parse` accepts what it accepts today,
and the plan reports the name with the diagnostic `name-unportable`, naming
the rule, so the user can rename it. When plans can propose renames
(testing.md, "Still open"), an NFD name read on macOS is shown as a
normalization to NFC, not as a different label.

### 5. Placement compares names as macOS and Windows do

ADR 0005's `placement-collision` ("two entries resolve to the same path")
compares paths exactly. Under `portable` it compares each path component by
a key: NFC, then Unicode default case folding. Two paths with the same key
are one folder on macOS and Windows, so they collide. Default case folding
approximates both systems' tables (NTFS uses its own upper-case table); where
they differ, xD reports a collision that one of them might allow, never the
other way round.

Whole paths have a limit only once a root is known, so the core's `check`
(ADR 0003) adds `path-too-long`: a planned path whose length, from the actual
root, exceeds Windows' 260 characters. It is a warning, since Windows can
enable long paths, and the planner runs before any file is touched.

The same contracts apply to the `dir` part of ADR 0008's locations, which is
a path xD writes.

### 6. Tests

Following docs/testing.md:

- **Examples** in `nickel/tests/`: one accepted and one rejected name per
  rule, each naming the source it comes from (`CON`, `nul.txt`, `Taxes.`,
  `Taxes `, `a:b`, a 256-byte name, a 255-byte one, an NFD `Café`).
- **Agreement**: the Nickel contract and the Rust check agree on random
  strings biased toward the edges: reserved names in any case with and
  without extensions, trailing dots and spaces, control characters, NFD
  strings, names near 255 bytes.
- **Law** in `crates/xd-conformance`: every name xD renders, with any label
  `portable` accepts, satisfies `portable`; and two names with different
  comparison keys never collide.
- **Disk properties** in `crates/xd-core/tests/folders.rs`: names `portable`
  accepts are created in a temporary directory, listed and read back as the
  same name after NFC on every operating system; names with different keys
  are different folders. Each target's rejected examples are tried on its own
  system: where the system really refuses or changes the name (Windows
  stripping a trailing period), the test asserts it; where a system has
  relaxed a rule, the test records that in the target's data rather than
  failing.
- **CI**: a **Windows job** (`windows-latest`, NTFS) joins the Linux and
  macOS jobs. Nix doesn't run on Windows, so it installs Rust with rustup and
  runs `cargo test`. If `xd-nickel` (Nickel's interpreter through
  `nickel-embed`) doesn't build there, the job starts with
  `cargo test -p xd-core -p xd-conformance` and the gap is recorded in
  testing.md.

## Consequences

- Every rule xD enforces on a name points at a published source, and fixing
  a rule is a data change.
- Labels that work on Linux but not Windows (`Taxes: 2025`, `Notes?`) are
  rejected by default. That is the cost of `portable` being the default; the
  deferred `file_system_targets` capability is the way out once someone needs
  it.
- Existing folders keep working: decision 4 reports instead of failing.
- CI gains a Windows job, so the folder tests check Windows itself rather
  than a list of its characters.
- This changes no sealed rule in extensibility.md: labels were never
  identity, and the number space is unchanged.

## Open

- Whether `path-too-long` should use 260 characters or a configurable
  budget, once someone hits it with long paths enabled.
- Whether `portable` should also exclude characters that sync services
  (OneDrive, Dropbox) or archive formats reject, which are stricter than the
  file systems themselves. Not until a user reports one.
- Whether names should avoid a leading space or a leading `-` (legal, but
  awkward in shells). Not rules of any file system, so left out for now.
- If ADR 0008 isn't accepted, `nickel/targets.ncl` still holds the three
  file-system entries on its own.
