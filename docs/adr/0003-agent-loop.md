# 3. The agent loop: plans in, checked receipts out

Status: proposed, 2026-11-12.

## Context

ADR 0002 fixed plans as data, check, apply and undo, but left open the first
set of operations and how a plan reaches xD. The design in
[designs/agent-loop.md](../designs/agent-loop.md) (approved, then
eng-reviewed; decisions D15–D24 in its ledger) answers both for v1: an agent,
not xD's own planner, writes the plan. It reads a scan of a messy folder,
emits a plan as JSON, and xD checks, applies and undoes it.

v1 has one writer per system, so it skips ADR 0002's three-way
reconciliation (accepted shortcut `99191eb3`): hand edits on disk aren't
detected and the config can lag the disk until `RecordInConfig` is emitted
reliably. Upgrade when a second writer appears or spark-joy needs
desired-state input.

## Decisions

### 1. Commands

All print JSON on stdout. Errors are JSON with a finding code and exit
nonzero.

| Command | Does |
| --- | --- |
| `xd init <dir>` | Writes the config, creates `00-09 System` (config, `entries.json`, `00.00` JDex, a jj repository rooted there) and `.xd/`. Not undoable. Every other command refuses a folder that isn't a system. |
| `xd scan <dir>` | Read-only walk; prints the aLDex (path, kind, size, mtime, content hash, symlink target, `hidden`, `error`). Skips `.xd/` and `00-09 System`. |
| `xd check <plan.json>` | Loads state, re-scans, runs `check`; prints findings, or a receipt (the plan's hash). |
| `xd apply <plan.json> --receipt <hash>` | Refuses a mismatched receipt, re-checks against current state and a fresh scan, applies a `Plan<Checked>` (ADR 0002 decision 4). |
| `xd undo` | Inverts the newest applied plan not yet undone and applies it like any plan. One step per call; no redo. |
| `xd recover` | Rolls back an apply that was cut short. |

`xd init` and `xd apply` check that `jj` is on `PATH` before writing anything
and stop if it isn't. `scan` and `check` don't need jj. The flake ships it.

### 2. Operations

v1 takes `CreateFolder`, `RemoveEmptyFolder`, `Rename`, `Move`,
`WriteJdexEntry` and `RecordInConfig` from ADR 0002, and adds one pair for
files that have no address yet:

| Operation | Inverse |
| --- | --- |
| `Adopt { from_path, to_address, name, size, hash }` | `Release { from_address, name, to_path }` |

`Adopt` moves an unaddressed file or folder, whole, into an ID folder. Its
precondition is the size and hash from the scan (mtime too for files; not
for folders, whose mtime the same plan may change).

Each operation's effect on the Registry is fixed:

| Operation | Registry |
| --- | --- |
| `CreateFolder` | `insert` |
| `RemoveEmptyFolder` | `remove` (new) |
| `Rename` | `rename` |
| `Move` | `insert` the destination, `remove` the source |
| `Adopt`, `Release`, `WriteJdexEntry` | none |

`Registry::remove` succeeds only for an active entry with no children, and
frees the number.

**A move is not a retirement.** Only archiving retires a number: the entry
is finished, and its number is never used again. A move is the same entry at
a new number, so the old number is simply free. This replaces the line in
jd-formal 1.4 that had renumbering retire `n`. No forwarding note is left at
the old number; the plan's `WriteJdexEntry` steps move the JDex entry, and
undo reverses them with the rest. Undo needs nothing special: a
`Move` back is just another `Move`, and undoing a `CreateFolder` frees its
number. "Retired only grows" holds without exception.

### 3. Check reuses the Registry

`check(plan, &Registry, &Scheme, &Aldex) -> Vec<Finding>` is pure, in
`xd-core`. It applies the plan's entry operations to a clone of the Registry
and maps each error to a finding; it has no number rules of its own.

| Finding | From |
| --- | --- |
| `duplicate-number` | `RegistryError::Duplicate` |
| `reused-number` | `RegistryError::Reused` |
| `not-closed` | `RegistryError::MissingParent` |
| `capacity` | `RegistryError::Invalid` / `Full` under the loaded scheme and its reserved set |
| `not-empty` | a folder to remove isn't empty in the aLDex (lists the files) |

Capacity is the scheme's, never JD's numbers: with the JD defaults and
standard zeros that's 9 areas, 9 categories per area and 89 IDs per category
(`.11`–`.99`), and another scheme gets its own limits for free.

The path rules, which the Registry can't see:

| Finding | When |
| --- | --- |
| `outside-root` | a write resolves outside the system root, including through a symlink target in the aLDex |
| `not-scanned` | an `Adopt` source isn't in the aLDex, is under `.xd/` or `00-09 System`, or has an `error` |
| `not-an-id` | an `Adopt` target isn't an active ID-level entry |
| `name-clash` | two destinations in one folder are equal after case folding and Unicode normalization |
| `nested` | one `Adopt` source is inside another |
| `no-inverse` | an operation has no inverse and isn't marked irreversible |

### 4. Where state lives

| What | Where | Written by |
| --- | --- | --- |
| Scheme | the hand-written Nickel config in `00-09 System` | the user |
| Live and retired entries | `00-09 System/entries.json`, imported by the config | `xd-cli` only (the one config writer of ADR 0002) |
| Applied plans and undo markers | `.xd/plans/<plan-id>.json` | `xd-cli` |
| Journal | `.xd/journal/<plan-id>.jsonl` | `xd-cli` |

`xd-nickel::load` returns the scheme and a Registry built from the imported
entries, so Nickel contracts validate them; a config that imports no entries
loads an empty Registry, and an entry both live and retired fails to load.
`entries.json` is written to a temporary file and renamed into place.

`.xd/plans/` alone decides which plan `xd undo` picks. The
`xd-applied/<host>` bookmark keeps only its ADR 0002 role: the configuration
as of this machine's last apply.

### 5. Apply and recover

Apply, in order:

1. Save the plan to `.xd/plans/`.
2. For each operation: journal it, check its precondition, run it, mark it
   done.
3. Write `entries.json` and the JDex, then commit in jj and move the
   bookmark. These are the last journal steps.

Only an `Adopt` source is hashed during apply, just before it moves; the
re-scans in check and apply record size and mtime, not hashes. Moves use a
rename that refuses to replace (`renameat2(RENAME_NOREPLACE)` on Linux,
`renamex_np(RENAME_EXCL)` on macOS) and fail on `EXDEV`; xD never copies
and deletes. Precondition failures are `stale` (size, mtime or hash
changed), `collision` (destination exists) and `missing` (source gone).
On any failure apply runs the inverses of the done steps in reverse (ADR
0002 decision 5.2).

If a command finds a journal with an open step, it refuses and points to
`xd recover`. Recover looks at the disk for the open step (source or
destination present?), inverts it if it ran, then inverts the done steps in
reverse. It rolls back; it never resumes.

### 6. Code layout and dependencies

- `xd-core`: a `plan` module (operations, invert, check), plus
  `Registry::from_entries` and `Registry::remove`. Still no I/O.
- `xd-nickel`: `load` gains the entries.
- `xd-cli` (new): `main.rs`; `system.rs` (finding the system, `.xd/`,
  journal, plans, recover, jj); `disk.rs` (scan, the directory backend).
- Content hash: SHA-256 through `sha2`, already in the dependency tree via
  Nickel, so no new crate is built. Hashes are prefixed `sha256:` so the
  algorithm can change.
- No-replace rename: `rustix`, also already in the tree. Its macOS support
  for the exclusive rename is to be confirmed when `disk.rs` is written; if
  it lacks it, call `renamex_np` through `libc` (also in the tree).

## Consequences

- An agent can organize a folder end to end with no planner in xD: `init`,
  `scan`, write a plan, `check`, `apply`, and `undo` if it went wrong.
- Number rules live in one place, the Registry; check and apply can't
  disagree about them.
- xD depends on the `jj` binary at runtime.
- Undo can be refused (`not-empty`) when the user has saved into a folder
  since; it never moves files it wasn't told to.
- The tests are listed in the eng-review test plan: laws in
  `xd-conformance` (invert twice is the identity; a plan and its inverse
  restore the Registry), the rest as tempdir tests in `xd-cli`, including
  crash injection after each journal step.

## Open

- aLDex format (JSON Lines or one document) and its fields beyond the above.
- How `xd scan` reports files already named with a JD address.
- Whether duplicates (same hash) are reported in the aLDex.
