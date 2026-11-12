# 2. Plan and apply: three-way reconciliation, plans as data

Status: proposed, 2026-11-12.

## Context

ADR 0001 (decision 3) says each command reads the configuration, the disk
and the JDex into one model, a pure function turns the difference into a
plan, and backends carry it out. It doesn't say how the difference is
computed, what a plan is, or how undo works. [design-patterns.md](../design-patterns.md)
compares the options; this record fixes them.

Requirements carried over from the earlier design work:

- **Either side can change.** A *config edit* changes the Nickel files; a
  *disk edit* changes folders by hand; a *command* (`xd new`, `xd retire` …)
  writes the config through the one config writer and then plans as usual.
- **The configuration never deletes or renumbers on its own.** Retiring is
  explicit, and a renumber is declared (`from = "11.11"` on the new entry).
- **History is jj.** The configuration and the JDex live in a jj repository.
  A per-machine bookmark marks the configuration as of that machine's last
  apply; it was called `jd-applied/<host>` and becomes `xd-applied/<host>`
  (not `@<host>`, which is jj's remote syntax).
- **Nothing is applied that the user hasn't seen.**

## Decisions

### 1. Three inputs, not two

A plan is computed from:

| Input | Where it comes from |
| --- | --- |
| **Desired** | the configuration at the working copy |
| **Observed** | the folders, the JDex and the apps, read now |
| **Last applied** | the configuration at `xd-applied/<host>` |

With only desired and observed, a folder that exists on disk but not in the
configuration is ambiguous: removed from the configuration, or made by hand?
The last-applied state decides it, as `kubectl apply` uses
`last-applied-configuration`. For each entry:

| Desired vs last applied | Observed vs last applied | Meaning | Plan |
| --- | --- | --- | --- |
| same | same | nothing changed | nothing |
| changed | same | config edit | change the disk |
| same | changed | disk edit | propose recording it in the configuration |
| changed | changed, to the same result | both agree | nothing |
| changed | changed, differently | conflict | stop; show both; change nothing |

A disk edit is never undone silently. Recording it is itself a plan the user
accepts, or the user restores the disk.

On a first run there is no `xd-applied/<host>` yet: last applied is treated
as equal to observed, so everything on disk counts as already there and only
config edits produce changes.

The comparison is **level-triggered**: every `xd plan` recomputes from the
three states, rather than replaying watcher events, so a missed event can't
make a plan wrong. A watcher only tells xD *when* to plan.

### 2. A plan is a list of operations, and nothing else

Planning is a pure function in the core:
`plan(desired, observed, last_applied) -> Plan`. A plan is an ordered list
of operations, each a plain, serializable value, for example:

| Operation | Inverse |
| --- | --- |
| `CreateFolder { address, name }` | `RemoveEmptyFolder { address, name }` |
| `Rename { address, from, to }` | `Rename { address, from: to, to: from }` |
| `Move { address, from, to }` (renumber, archive) | `Move` back |
| `WriteJdexEntry { address, before, after }` | `WriteJdexEntry` with `before` and `after` swapped |
| `RecordInConfig { address, before, after }` | the same, swapped |
| `SetStorage { address, before, after }` | the same, swapped |

Rules:

- **Every operation carries what's needed to invert it** (the `before`
  values). An operation without an inverse is marked irreversible, is
  listed separately in the plan, and needs its own confirmation. Content is
  never deleted by xD; retiring moves it (formal 2.5), so the first
  irreversible operations are expected only from remote storage backends.
- **Operations name entries by number** (the identity), and carry names only
  as data. A rename is never a delete plus a create.
- The set of operations will grow; the enum is `#[non_exhaustive]`.

### 3. Interpreters run plans

| Interpreter | Does |
| --- | --- |
| **show** | prints the plan (`xd plan`) |
| **check** | verifies the plan against the formal invariants (no reused number, closure, one entry per number, no write outside the system's root) |
| **apply** | carries out each operation through the storage and app backends, journaling each step before it runs |
| **invert** | returns the plan that undoes a plan |
| **simulate** | applies a plan to an in-memory tree, for tests |

Hooks (extensibility.md) receive a plan and return a plan, so their output
goes through check like any other plan and stays undoable.

### 4. Only checked plans can be applied

The stages are types: `Plan<Draft>`, then `check()` returns
`Plan<Checked>` or the violations, and `apply()` accepts only
`Plan<Checked>`. Skipping the check is a compile error.

### 5. Apply, record, undo

1. `xd apply` shows the plan and asks (or takes `--yes`).
2. It journals and runs each operation in order. If one fails, it runs the
   inverses of those already done, in reverse order, and reports.
3. On success it commits the configuration and JDex changes in jj and moves
   `xd-applied/<host>` to that commit.
4. `xd undo` inverts the last applied plan, applies that, and moves the
   bookmark back. Changes to the configuration and JDex are also undone by
   `jj op undo`; folder changes need xD's inverse, because jj doesn't track
   the tree.

## Consequences

- The core gains a `plan` module (three-way diff, operations, typestate) and
  a test interpreter; it still does no I/O.
- `xd plan` is safe to run at any time, including from a watcher or the xjd
  mod.
- Conflicts stop the run instead of guessing. Resolving them needs a small
  UI (keep config, keep disk, or edit), designed when `xd plan` exists.
- The bookmark makes "last applied" per machine, which shared systems need
  later (each machine reconciles its own disk against shared configuration).

## Open

- The exact first set of operations, settled while writing `plan`.
- What "observed" includes from apps (Org JDex, task managers) before their
  adapters exist: for now, folders and the JDex only.
