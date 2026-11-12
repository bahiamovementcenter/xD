# Declarative config vs jd CLI: workflow rules

Nov 11, 2026 · Ash. Exported from the live doc: https://claude.ai/artifact/WAJtDtQhqdfsYjc83Ga5wv (edit there, then re-export).

## Summary

The jd CLI and the Nickel config can share one Johnny.Decimal tree if both follow five rules.

1. **The number is the identity; the name is a label.** `10-19`, `11`, `11.11` and `W0012` are what everything matches on, as jd already does. A new name on the same number is a rename, never a new thing.
2. **Both sides can change anything, and every change ends up in the config.** You edit the config; jd commands change the disk and write the same change to the config through `jdncl`. `jd apply` never reverts what a command did.
3. **Nothing reaches the disk without a plan.** A config edit goes through `jd plan`, then `jd apply`. A jd command is its own plan.
4. **Undo follows the change.** jj keeps the history of the config and the JDex. `jd undo` reverses the newest change on disk and in the config together.
5. **Nothing is deleted or renumbered silently.** Removing an entry leaves it on disk with a warning. `archived = true` retires its number for good. Renumbering is always explicit, with `jd renumber` or `from`.

Every change is one of three kinds, named by where it starts:

| Term | What it is | How it reaches the rest |
| --- | --- | --- |
| **Config edit** | You change `meta-system.ncl`, a file it imports, or a `.jd.ncl` | `jd plan`, then `jd apply` |
| **Command** | `jd new`, `jd rename`, `jd archive`, `jd renumber`, `jd move`, `jd undo` | The command changes disk, JDex and config in one step |
| **Disk edit** | Anything else that changes a structure folder or JDex entry: `mv`, a file manager, a sync tool, a script, a heading edited in Emacs | `jd plan` reports it and offers a step that records it; `jd apply` runs that step |

Errors stop a command and change nothing. Warnings print and exit 0, and become errors under `jd check --strict`.

## How it works today

As of draft PR #5 and jd 5.0.1, the config only creates things, and jd's only undo is for `jd move`.

| Layer | What it changes | Matches by | Undo |
| --- | --- | --- | --- |
| `system.ncl` + `jd scaffold` | Creates declared areas, categories and IDs (folder + JDex note or Org heading). Never deletes. | Full name, `10-19 Life admin/11 Me` | None |
| `jd check` | Nothing. Reports `missing` and `undeclared` | Full name | n/a |
| `jd new id` / `jd new wp` | An ID or work package: JDex entry, folder, task project | Number in the JDex | None |
| `jd move` | Moves a file or folder into an ID or W folder, optionally renamed with `--as` | Number | `jd undo move`, from the journal |
| Disk edits | Anything | n/a | None |

What matters from jd 5.0.1 ([README](https://github.com/johnnydecimal/cli.johnnydecimal.com/blob/v5.0.1/README.md)):

- The journal, `~/.local/state/johnnydecimal/journal.jsonl`, is append-only, per machine, and stores **absolute paths**. `jd undo move` fails with `moved_since` once the thing is no longer at its `to` path, so renaming any folder above a moved file strands that move.
- jd has no rename, archive or delete command. Archiving is moving into `.09` or `W0009`, and jd counts what is there, so archived numbers are never handed out again.
- The JDex defines the IDs. A folder with no JDex entry is not an ID to jd. IDs start at `.11`.
- jd finds an ID folder named `11.11` or `11.11 …`, and a work package named `W0012`, `W0012 …` or `W0012~…`. Anything else is skipped.
- Errors carry a stable `code` (`--json` gives `{ok, code, message, path, warnings}`). Our commands use the same shape.

The gap: `jd scaffold` and `jd check` match on full names, so renaming a category in `system.ncl` looks like "one missing, one undeclared" and scaffold makes a second `11` folder. And `jd check` flags every `jd new id` ID as `undeclared`, as if it were a problem.

This design also renames `system.ncl` to `meta-system.ncl`, `jd.project` to `jd.meta-system` and `JD_SYSTEM` to `JD_META_SYSTEM`, with no aliases, since nothing is deployed yet.

## Who can change what

Either side can create, rename, archive or renumber any part of the structure. What differs is how the change reaches the other side.

| Change | Disk and JDex | Config |
| --- | --- | --- |
| Config edit | `jd plan`, then `jd apply` | already there |
| Command (`jd new`, `jd rename`, `jd archive`, `jd renumber`, `jd undo`) | done by the command | written by `jdncl` in the same step |
| `jd move` of files | done by jd | not recorded: the config describes structure, not contents. A `+ REF` folder it creates is structure, so `jd plan` offers to record it |
| Disk edit | already there | central layout: `jd plan` reports it and offers a step that records it. Distributed layout: the folder name already is the config, and the plan brings the JDex in line |

### `jdncl`: the one writer of the config

One tool, `jdncl`, is the only thing that edits `meta-system.ncl`, its imports and the `.jd.ncl` files. Every jd command that changes structure calls it, and you can call it yourself: `jdncl set 11.12 name Taxes`, `jdncl set 11.13 archived true`, `jdncl show 11.12`.

- It finds the one file that defines the number (central or dotfile) and edits only that entry, keeping your formatting and comments. A number defined in two places stops it with `ambiguous_source`.
- In the distributed layout, names aren't stored in files, so a rename there is a folder rename, not a `jdncl` edit.
- It evaluates the result with Nickel before writing. If the contracts fail, nothing is written and the command that called it stops with `config_unwritable`.
- jj records every write, so a `jdncl` edit is undone like any other change.

### Plan and apply

The applied record is a jj bookmark per machine, `jd-applied/<host>` (for example `jd-applied/laptop`; not `@`, which jj uses for remotes). Each `jd apply` moves it to the change it applied, so the config at that bookmark is what this machine's disk was last made to match. One per machine is right because each machine has its own disk: a rename pulled from another machine shows up as a config edit, and the plan renames the folder here too.

Every check compares three names for each number: **C** the config in the working copy, **A** the config at this machine's bookmark, **D** the disk (folder and JDex entry).

| C vs A | D vs A | Meaning | Result |
| --- | --- | --- | --- |
| same | same | in sync | ok |
| changed | same | config edit, here or pulled | the plan renames on disk |
| same | changed | disk edit | warning `renamed_on_disk`; the plan offers a `jdncl` step that records the new name |
| changed | changed, D = C | both made the same change | ok; the next apply moves the bookmark |
| changed | changed, D ≠ C | both changed it, differently | error `rename_conflict`, nothing applied |
| same | gone from disk | deleted by hand | warning `deleted_on_disk`; the plan offers to recreate it or archive it, and skips it by default |
| no record | D ≠ C | first apply on this machine | error `unknown_baseline`; rerun with `--prefer config` or `--prefer disk` |

In the distributed layout C and D are the same folder names, so this table mostly matters for the JDex: if the folder and its JDex entry disagree, whichever changed since the bookmark wins and the plan updates the other (`jdex_name_mismatch`).

- `jd plan` reads config, record and disk, and prints the steps grouped as create, rename, move, archive and record (writing a disk edit into the config), with the warnings. Steps that undo can't fully reverse are marked one-way (see App entries below). It saves the plan to `.jd/plan.json` with a hash of the config and of the disk it was made from. Apart from that file it changes nothing.
- `jd apply` runs only the plan that `jd plan` saved, never one of its own. No saved plan: error `no_plan`. Config or disk changed since: error `stale_plan`, run `jd plan` again. It refuses a plan with any error, and asks before renames, moves and one-way steps unless `--yes`. Afterwards it labels the jj change and moves `jd-applied/<host>`.
- `jd check` is the same three-way report without saving a plan; `--strict` turns warnings into errors for CI and pre-commit.
- `jd new`, `jd rename`, `jd archive`, `jd renumber`, `jd convert` and `jd undo` are commands: each changes disk, JDex and config in one step, because the command you typed is the plan. `--dry-run` shows it first. They stop with `pending_creates` or `rename_conflict` when the config has unapplied edits that touch the same numbers.
- `jd scaffold` goes away: `jd plan` and `jd apply` cover what it did.

### Where each check runs

| When | Checks | Why there |
| --- | --- | --- |
| Nickel evaluation (`nix flake check`, every jd command) | The config alone: number rules, `from` and `archived` numbers reused, two entries on one number, work packages without an ID | Pure: Nix can't see the disk, so drift can never be checked here |
| `jd plan`, `jd check`, `jd apply` | Config vs record vs disk, storage backends, the JDex | Needs the disk |
| The `jd new` wrapper (around `_jd_new`) | A number the config declares but hasn't applied yet | The only moment the collision can be stopped |
| `jd rename`, `jd archive`, `jd renumber`, `jd undo` | Journaled moves that a rename would strand, undo order, backends that are offline | Before the change, not after |
| Shell start (`nix develop`, direnv) | One line: "3 warnings, run jd check" | Cheap nudge, never blocks |

## Workflows

Each row says what should happen and what the person sees. "Config edit" always means: edit, `jd plan`, `jd apply`. Codes are in the catalogue below.

### Areas and categories

**Archiving has four strategies** (Ash, 2026-11-11). The system sets one with `archive = { strategy = … }`; an area, category or single entry can override it. Whatever the strategy, an archived number is retired and never reused, and its JDex entry stays (so jd's own numbering never hands it out again).

| Strategy | ID | Category | Area | Trade-off |
| --- | --- | --- | --- | --- |
| `'standard` (default) | into `11.09` | into its area's `10.09` | into `00.09` | The Johnny.Decimal standard zeros; costs no slots; needs the `x0` and `00` categories, which the plan creates if missing |
| `'last_slot` | into `11.09` | into its area's `19 Archive` | into `90-99 Archive` | Easy to see in a file manager; reserves `x9` in every area and `90-99` (`reserved_number`) |
| `'in_place` | stays | stays | stays | Nothing moves, so no paths or journaled moves break; `archive.storage` can move the contents to cold storage (e.g. `'rclone` to S3). Archived folders stay among live ones |
| `'separate_root` | same path under `archive.root` | same | same | For example `~/P76 Archive/10-19 Life admin/11 Me/11.13 Car`. Keeps the live tree clean; the system lives in two places |

Work packages go into `W0009` under `'standard` and `'last_slot`, and follow the other two like IDs. Changing the strategy is a config edit like any other: the plan moves what's already archived to its new place, and `strands_moves` warns if that breaks journaled moves.

| Workflow | What happens | Thrown |
| --- | --- | --- |
| Add category `13 Car` in the config | The plan lists the creates: the folder, the JDex entry or Org heading, and its standard zeros. Apply makes them | none |
| Rename `11 Me` to `11 Myself` in the config | The plan shows one rename, not a create. Apply renames the folder, the JDex entry and the Org heading text in place. Files inside move with it | warning `strands_moves` if journaled `jd move`s land under it |
| Rename the folder `11 Me` by hand | Central: drift, and the plan offers to record the new name. Distributed: that is the rename, and the plan updates the JDex | `renamed_on_disk` (central), `jdex_name_mismatch` (distributed) |
| Rename it on both sides, differently | Nothing applied | error `rename_conflict` |
| Move category `11` into area `20-29` | Not a rename: the number says it belongs in `10-19` | error `category_outside_area` at evaluation (exists today) |
| Renumber `11` to `14` | `jd renumber 11 14`, or `"14" = { name = "Me", from = "11" }` in the config. The folder moves, every `11.xx` becomes `14.xx`, linked work packages `W…~11.xx` are renamed, and `11` is retired. Removing `11` and adding `14` without `from` is ambiguous | `number_conflict` if `14` exists; `renumber_not_supported` without `from` |
| Remove category `12` from the config | Nothing deleted on disk | warning `orphaned` until you put it back or archive it |
| Archive category `12` | `jd archive 12` or `archived = true`: the folder goes where the strategy says, its JDex entries with it, and the number is retired | `open_work_packages` if work packages linked to `12.xx` aren't archived; `number_retired` if `12` is reused; `backend_unavailable` if `'in_place` cold storage is offline |
| Archive area `10-19` | As for categories, one level up | as for categories |
| Declare `19` or `90-99` for something else under `'last_slot` | They're reserved for the archive | error `reserved_number` at evaluation |
| Delete the folder `12 House` by hand | The JDex still has `12`, so nothing reuses the number. The plan offers to recreate it or archive it | warning `deleted_on_disk` |
| Make a category folder by hand | jd still reads it. The plan offers to record it (central) or writes its `.jd.ncl` (distributed) | warning `undeclared` |

### IDs

| Workflow | What happens | Thrown |
| --- | --- | --- |
| Declare `11.12 Tax` in the config | Apply creates the folder and JDex entry | none |
| `jd new id 11 Car` | jd takes `11.13`; `jdncl` records `"11.13" = "Car"` (central) or writes its `.jd.ncl` (distributed) | none |
| `jd new id 11 Car` while `11.12 Tax` is declared but not applied | jd would take `11.12`. The wrapper stops and tells you to run `jd plan` and `jd apply` first, so jd then picks `11.13` | error `pending_creates` |
| Rename an ID in the config | Apply renames the folder, the JDex note and the Org heading | `strands_moves` if relevant |
| `jd rename 11.12 Taxes` | Renames the folder and JDex entry, and `jdncl` writes the name | `strands_moves`; `rename_conflict` if the config has a different unapplied name |
| `jd archive 11.13` | The folder moves into `11.09` (a `jd move`, so in jd's journal), the JDex note or Org subtree goes under `11.09`, and `jdncl` sets `archived = true` | `archive_partial` if the folder moved but the note didn't; `open_work_packages` |
| Move `11.13` to category `12` | IDs don't move; the number is the place. `jd renumber 11.13 12` takes the next `12.xx`, moves the folder and its `+` folders, renames linked work packages, and leaves a JDex stub `11.13 → 12.11` under `11.09` so jd never reuses `11.13`. `jdncl` writes `"12.11" = { name = "Car", from = "11.13" }`, and `from` alone retires `11.13` | none. The same as a removal plus an addition in the config, without `from`, is `renumber_not_supported` |
| Two folders with one number, `11.12 Tax` and `11.12 Car` | Nothing can be matched safely | error `duplicate_number` |
| Delete `11.13 Car` by hand | As for categories | warning `deleted_on_disk` |

### Work packages

Work packages belong to the system, not to an area: they live in `W0000-9999` and are declared at the system level.

| Workflow | What happens | Thrown |
| --- | --- | --- |
| `jd new wp` | The number comes from `work_packages.first_number` (W0011), skipping W0003 and W0009 (PR #5). `jdncl` records it | none |
| Declare `W0012` in the config | Apply creates it as `jd new wp` would, task project included. The step is one-way if the task adapter can't remove projects | `number_conflict` if a different `W0012` exists |
| Archive `W0012` | `jd archive W0012`, or `archived = true` in the config: the folder and its `+` folders move into `W0009`, the JDex entry goes under `W0009` | `archive_partial` |
| Rename `W0012` | `jd rename` or a config edit: the folder, the JDex entry, and the task project if the adapter can rename | `strands_moves`; `app_entry_left` if the project keeps its old name |
| Undo `jd new wp` | Removes the folder and JDex entry, and the task project if the adapter can remove it | `app_entry_left`, with the project's link, if it can't |
| `W0003` or `W0009` missing | The plan recreates them | none |

### Extend-the-end `+ REF` folders

A `+ REF` folder sits next to its ID in the category folder, named `11.11+ W0012` (no name between the number and `+`). It sorts right after `11.11 Passport`, because a space sorts before `+`, and `+` before the next digit. Leaving the name out matters: jd finds an ID by `11.11` or `11.11 *`, so `11.11 Passport+ W0012` would be a second `11.11` folder (`duplicate_number`), while `11.11+ W0012` is skipped by jd.

| Workflow | What happens | Thrown |
| --- | --- | --- |
| Rename the parent ID | Nothing to do: the `+` folder carries no name | none |
| Renumber `11.11` to `11.12` | The plan renames every `11.11+ *` to `11.12+ *` with the ID | `number_conflict` if a `11.12+ REF` already exists |
| Archive `11.11` | The plan moves its `+` folders into `11.09` with it | `archive_partial` if some moved and some didn't |
| `11.11 Passport+ W0012` (name before `+`) | Looks like a second `11.11` to jd | error `duplicate_number`, with the fix `11.11+ W0012` |
| `11.13+ X` with no `11.13` |  | warning `dangling_ref` |
| REF names an archived ID or W number | Still valid | warning `ref_to_archived` |
| Declare one in `meta-system.ncl` | Allowed: `"11.11+ W0012" = {}` under the category, the contracts accept `+` | none |

Work packages extend the same way: `W0012+ REF` sits next to `W0012~11.11 Website` in the work package area. jd finds a work package by `W0012`, `W0012 *` or `W0012~*`, so it skips `W0012+ …`, and a `W0012+` entry in the JDex doesn't change the next free number. One difference: `+` sorts before `~`, so for a work package tied to an ID the `+` folder would show just above it; the naming rule below fixes that. Archiving `W0012` moves its `+` folders into `W0009` with it, and renumbering renames them, as for IDs.

Every work package names its ID: `jd new wp` needs one (error `no_id` without it), and the folder, JDex entry and Org heading are `W0012~11.11 Title`. So renumbering `11.11` to `11.12` also renames each linked `W…~11.11` to `~11.12`, and archiving `11.11` while it has work packages outside `W0009` warns `open_work_packages`.

The `+` folder itself is `W0012+ REF`: a W number is unique, so it needs no ID. To keep it below its work package, the work package folder's `~` becomes a space while it has `+` folders (Ash, 2026-11-11):

| State | Work package folder | Sorts as |
| --- | --- | --- |
| No `W0012+` folders | `W0012~11.11 Website` (jd's own name) |  |
| One or more `W0012+` folders | `W0012 11.11 Website` | `W0012 11.11 Website`, `W0012+ Brief`, `W0013~…` |

- The plan for creating the first `W0012+` folder includes renaming `W0012~` to ` W0012  `; archiving or removing the last one renames it back. jd finds both names (`W0012 *` and `W0012~*`).
- Only the folder changes. The JDex entry and Org heading keep `~`.
- Each switch is a rename, so it can strand journaled `jd move`s into the work package (`strands_moves`).
- `W0012 11.11 Website` with no `+` folder, or `W0012~…` next to one, is a warning `wp_name_form` that the next plan fixes.
- `W0012~11.11+ REF` stays invalid: jd reads it as a second `W0012` (`duplicate_number`).

**No bare work packages.** A work package folder must carry its ID: `W0012~11.11 …` or, with `+` folders, `W0012 11.11 …`. A folder named `W0012` or `W0012 Website` (no `AC.ID`) is an error `wp_without_id` in check and plan, and `jd apply` refuses to run until it is renamed. In the config, a declared work package without `id` fails evaluation with the same code.

### Org JDex (PR #5)

| Workflow | What happens | Thrown |
| --- | --- | --- |
| Rename in `meta-system.ncl` | Apply rewrites only the heading text after the number; body, properties and `CUSTOM_ID` stay | none |
| Rename a heading in Emacs | Same as a hand rename of a folder | warning `renamed_on_disk` |
| The Org file changes while apply runs (unsaved Emacs buffer saved over it) | Apply checks the file's hash before writing | error `jdex_changed`, nothing written |
| Two headings with one number |  | error `duplicate_number` |
| Links like `[[#W0012]]` | Survive every rename, because `CUSTOM_ID` is the number | none |

### App entries

A work package also has an entry in the task app (a Things project, an Org TODO) and a note in the JDex. Upstream jd's adapters can create these but not remove or rename them: the interface is `create`, `link` and `refresh`. The apps themselves can. Things deletes a project through AppleScript, into its own Trash; Bear has a trash action; Org and Markdown entries are text.

So we add optional adapter functions: `jd_tasks_remove`, `jd_tasks_rename`, `jd_notes_remove`. When an adapter has them, undo and rename reach the app too. When it doesn't, `jd plan` marks the step one-way, and undo or rename warns `app_entry_left` with the entry's link so you can fix it by hand.

### Storage backends

Any area, category, ID or work package folder can have its own storage: a repository, a path outside the root, cloud storage, a mount. The parent declares it, because a mounted folder can't hold its own `.jd.ncl`: it would vanish whenever the mount is down.

```
"11.11" = { name = "Website", storage = { kind = 'repo, url = "git@github.com:me/site.git" } },
"11.12" = { name = "Photos",  storage = { kind = 'rclone, remote = "b2:photos", encrypt = true } },
```

| Kind | The folder is | jd's part |
| --- | --- | --- |
| `'dir` (default) | a plain folder on this machine | create it |
| `'path` | a symlink to a path outside the root, including an NFS or SMB share the system mounts | create the link |
| `'repo` | a clone; `tool = 'git` (default) or `'jj`, `annex = true` for git-annex | clone on first apply, check the remote URL; never commits |
| `'subvolume` | a btrfs subvolume or ZFS dataset, for its own snapshots, quota and compression | create it |
| `'rclone` | an rclone mount: S3, Google Drive, WebDAV, SFTP and the rest | mount it (on NixOS a systemd user unit) |
| `'syncthing` | a Syncthing folder, peer to peer | register it; tell Syncthing when it's renamed or moved |
| `'fuse` | any other FUSE mount, given its command (for example btfs, a read-only torrent) | mount it |

`encrypt` works on any kind (gocryptfs, or rclone crypt for `'rclone`), because it wraps whatever is underneath. rsync isn't a kind: it copies one way. jd uses it to move data, for example when a folder with contents gets a backend.

**Sync the repo, not the root.** The config repo is the one thing that travels between machines. On each machine, `jd plan` and `jd apply` build the tree from the config and attach each folder's backend. A plain `'dir` folder's contents stay on the machine that has them.

| Workflow | What happens | Thrown |
| --- | --- | --- |
| Declare storage on a new ID | Apply creates it with its backend: clone, link, subvolume or mount | none |
| Add storage to a folder that has contents | jd won't clone into or mount over them. The plan offers a migration step: rsync into the backend, verify, then switch | error `backend_not_empty` without that step |
| Backend offline (mount down, remote unreachable) | An empty mount must never look like a deletion. The plan skips that folder and everything under it | warning `backend_unavailable` |
| The folder isn't what the config says (plain folder where it says `'repo`, wrong remote URL) | Nothing applied to it | error `backend_mismatch` |
| Rename, renumber or archive a backed folder | One planned step: unmount, move, update the unit or Syncthing, remount | as for any rename |
| `jd move` into a backed folder | Crosses filesystems, so jd copies, checks, then deletes, instead of `mv` | `backend_unavailable` if it's offline |
| On NixOS | The flake's home-manager module turns `'rclone`, `'fuse` and `'syncthing` entries into systemd user units, so mounts are declared like everything else | none |

## Config layouts

Both layouts are supported, per system, and can be mixed. The one rule: every number is defined in exactly one place.

### Central: everything in the repo

Names live in the config. A disk edit of a name is drift (`renamed_on_disk`). The root holds folders only and isn't synced; each machine builds it from the repo.

```
my-system/                         repo, in jj, pushed to a git remote
├── flake.nix
├── meta-system.ncl
│     let jd = import "johnny-decimal.ncl" in
│     jd.meta-system {
│       systems = [{
│         sys = "P76", title = "Personal", root = "~/P76",
│         jdex = { app = "org", path = "jdex.org" },
│         areas = { "10-19" = import "areas/10-19.ncl" },
│         work_packages = {
│           first_number = 11,
│           W0012 = { id = "11.11", name = "Renew" },
│         },
│       }],
│     }
├── jdex.org                       the JDex, tracked with the config
└── areas/
    └── 10-19.ncl
          {
            name = "Life admin",
            categories = {
              "11" = {
                name = "Me",
                ids = {
                  "11.11" = "Passport",
                  "11.12" = { name = "Website",
                              storage = { kind = 'repo, url = "git@github.com:me/site.git" } },
                },
              },
              "12" = {
                name = "Money",
                ids = { "12.11" = { name = "Taxes", from = "11.13" } },
              },
            },
          }

~/P76/                             folders only, built by jd apply
├── 10-19 Life admin/
│   ├── 11 Me/
│   │   ├── 11.09 Archive/         holds the 11.13 → 12.11 stub
│   │   ├── 11.11 Passport/
│   │   ├── 11.11+ W0012/
│   │   └── 11.12 Website/         a git clone
│   └── 12 Money/
│       └── 12.11 Taxes/
└── W0000-9999 Work packages/
    └── W0012 11.11 Renew/         a space, not ~, while W0012+ exists
```

### Distributed: a `.jd.ncl` in each folder

Names live only in folder names; a `.jd.ncl` holds everything else. A folder that doesn't exist yet is queued in its parent's `create`, which `jd apply` empties once it has made it.

- **Every structure folder has a `.jd.ncl`**, even if it's `{ }`; `jd apply` writes a missing one (`missing_dotfile`). The root is its own jj repo that tracks only the `.jd.ncl` files and the JDex, so a folder rename shows up in jj as its `.jd.ncl` moving. That is the rename history in this layout.
- **A backed folder can't hold its own `.jd.ncl`**, so its parent's `.jd.ncl` describes it, name included: the one case where a dotfile may set `name`.
- **Across machines** the root repo is pushed to a git remote like the central repo, never synced with a file-sync tool. A rename pulled from another machine arrives as a `.jd.ncl` in a new folder while the contents are still in the old one. `jd plan` recognises the pair (same number, one folder has only the dotfile, the other has the contents and lost its dotfile) and plans moving the contents across (`pulled_rename`).

```
my-setup/                          repo, in jj
├── flake.nix
└── meta-system.ncl
      let jd = import "johnny-decimal.ncl" in
      jd.meta-system {
        systems = [ "~/P76", "~/D25" ],      roots; each holds its own .jd.ncl
        default = "P76",
        extensions = { "stats.sh" = { commands = ["stats"] } },
      }

~/P76/                             its own jj repo: only .jd.ncl files and the JDex
├── .jd.ncl                        { sys = "P76", title = "Personal",
│                                    layout = 'distributed,
│                                    jdex = { app = "org", path = "jdex.org" },
│                                    work_packages = { first_number = 11 } }
├── jdex.org
├── 10-19 Life admin/
│   ├── .jd.ncl                    { }
│   ├── 11 Me/
│   │   ├── .jd.ncl                { create = { "11.13" = "Car" },
│   │   │                            "11.12" = { name = "Website", storage = { kind = 'repo, ... } } }
│   │   ├── 11.11 Passport/
│   │   │   └── .jd.ncl            { }
│   │   ├── 11.11+ W0012/
│   │   │   └── .jd.ncl            { }
│   │   └── 11.12 Website/         a git clone, described by its parent
│   └── 12 Money/
│       ├── .jd.ncl                { }
│       └── 12.11 Taxes/
│           └── .jd.ncl            { from = "11.13" }
└── W0000-9999 Work packages/
    ├── .jd.ncl                    { }
    └── W0012 11.11 Renew/
        └── .jd.ncl                { tasks = { app = "org" } }

~/D25/                             a second system, same shape
└── .jd.ncl                        { sys = "D25", title = "Work", ... }
```

Deleting a folder by hand takes its `.jd.ncl` with it, but jj saw the file, and the JDex still has the entry. So `jd plan` still reports `deleted_on_disk` and the number isn't reused.

### Mixing them

A system can be central with a few areas distributed, or the reverse: `areas = { "10-19" = 'distributed, "20-29" = import "areas/20-29.ncl" }`. `jd` walks the distributed folders and generates the index that imports their `.jd.ncl` files, since Nickel can't glob. `jd convert 10-19 --to distributed` (or `central`) moves an area from one to the other as one command.

| Code | Level | When | Why |
| --- | --- | --- | --- |
| `ambiguous_source` | error | evaluation, plan | A number is defined both centrally and in a `.jd.ncl`, or in two files |
| `name_in_dotfile` | error | evaluation | A `.jd.ncl` names a folder that isn't backed; in the distributed layout the folder name is the name |
| `stray_dotfile` | warning | check | A `.jd.ncl` in a folder that isn't an area, category, ID, `+` folder or work package |
| `missing_dotfile` | warning | check, plan | A distributed folder with no `.jd.ncl`; read as `{ }`, and the plan writes it |
| `pulled_rename` | info | plan | A rename pulled from another machine; the plan moves the contents to the new folder |

Nix can't see `.jd.ncl` files in the root, so in the distributed layout only `jd plan` and `jd check` validate them, not `nix flake check`.

## Undo and history

The history is kept by Jujutsu (jj), with no manual commits. jj snapshots the working copy on every jj command, and with its watchman trigger on every file change, so nothing has to remember to commit. jj stores its data in a git backend, so the repo pushes to any git remote, but it isn't git to use: no staging, no commit step, and every operation can be undone.

**What's in jj:** the config (`meta-system.ncl`, its imports, the `.jd.ncl` files) and the JDex (an Org file or Markdown notes, all text). Folder contents are not: they're large, often binary, and live in their storage backends.

| Event | What lands in jj |
| --- | --- |
| A command, or `jd apply` | The config and JDex change, then jd runs `jj describe -m "jd renumber 11.13 → 12.11 Car"` and `jj new`, so each is one labelled change |
| You edit the config or a JDex note | jj snapshots it; `jd apply` labels it later |
| You rename a folder by hand | Central: nothing until `jd plan` records it, dated when you apply. Distributed: the `.jd.ncl` move, at the next snapshot |

**`jd undo`** is one command for every kind of change.

- It takes the newest of: a jd operation in jj's operation log (a command, an apply, a `jdncl` edit) and an entry in jd's move journal. It says which one it will undo.
- For a jj operation, it restores the config and JDex to before it and, if the operation changed the disk, reverses that in the same step, so disk, JDex and config stay together. It is a command, so it's its own plan; `--dry-run` shows it.
- For a `jd move`, it calls `jd undo move`.
- It stops with `undo_order` when something newer has to go first: a config edit you haven't applied, or a later rename above a moved file (jd would fail with `moved_since`).
- Before any rename, `strands_moves` counts the journaled moves under that folder, so the cost is known before it happens.
- Removing a work package's task project or note needs the adapter's remove function (see App entries); without one, undo warns `app_entry_left`.

jj's operation log is per machine, so `jd undo` only undoes what happened on this machine. To reverse a change made elsewhere, back it out with `jj revert`, then `jd plan` and `jd apply`.

**Reading:** `jd history 12.11` runs `jj log` over the changes that touch `12.11`, and `11.13` through its `from`.

**Merging:** jj records a conflict in the change instead of stopping. If two machines rename one number differently, both versions are kept, and `jd check` reports `rename_conflict` until it's resolved. Nothing else is blocked in the meantime.

## Shared systems

A whole system is shared or not; there's no sharing of single areas (Ash, 2026-11-11). A shared system has its own repo on a git host. Each member's `meta-system.ncl` lists it, and jd clones it next to the personal config and syncs it with `jj git fetch` and `jj git push`.

```
systems = [
  { sys = "P76", ... },                                  personal, as before
  { sys = "D25", title = "Work", root = "~/D25",
    shared = {
      remote  = "git@github.com:team/d25.git",
      forge   = 'github,                                  or 'forgejo, 'gitlab
      protect = true,                                     main only through reviewed pull requests
      review  = ['revert, 'renumber, 'archive],           which changes go as a pull request
      owners  = {
        "*"          = ["@ash"],
        "20-29"      = ["@finance"],
        "W0000-9999" = ["@ash", "@lee"],
      },
    },
  },
]
```

| Part | Shared or personal | How |
| --- | --- | --- |
| Config and JDex | shared | In the shared repo; anyone edits, jj merges. The Org JDex too, and a shared Emacs with crdt.el works |
| App choice and credentials | personal | `~/.config/jd/me.ncl` holds your user name, your task and note apps if they differ from the shared ones, and nothing else. Storage remotes are named in the shared config (`team-drive:`); each person's rclone config supplies the login |
| Applied record | personal | `jd-applied/<user>/<host>`, never pushed |
| Storage | shared | Shared folders need a shared backend: rclone (FTP, Dropbox, Google Drive, S3), Syncthing, a network share, BitTorrent through btfs or Resilio |

**Approval.** The git host enforces it, because jd runs on each person's machine and anyone could edit files by hand. The settings are all optional:

- `protect = true`: an admin's `jd share sync` turns on branch protection, so main only changes through reviewed pull requests.
- `review`: the kinds of change jd sends as a pull request (through `gh`, `tea` or `glab`). The rest are pushed straight to main, unless `protect` forbids it.
- `owners`: jd generates the host's CODEOWNERS file from it, mapping each area to the files that define it (`areas/20-29.ncl` centrally, the folder's `.jd.ncl` files when distributed). Per-area owners also need the JDex split per area, since CODEOWNERS works on whole files; with one Org file for everything, the `"*"` owners approve every JDex change.
- A change waiting for review exists only on your machine's branch. Nothing changes on anyone's disk, yours included, until it's merged and `jd plan` picks it up.

**Numbers.** Two people can take `11.13` while offline. When the changes meet, the plan keeps the one merged first and renames the other to `11.13+ <jj change ID>` (Ash's idea; the change ID survives rebases). Nothing is lost and there's no duplicate; `jd check` warns until `jd renumber` gives it a real number.

**Undo.** `jd undo` takes back your own last operation, from your machine's history. `jd revert <change>` reverses anyone's change by adding a new one, so nobody's history is rewritten; if `review` includes `'revert`, it goes as a pull request.

| Code | Level | When | Why |
| --- | --- | --- | --- |
| `number_collision` | warning | check, plan | A `+ <change ID>` holding folder from a collision; renumber it |
| `pending_review` | info | check, plan | Your change is waiting in a pull request |
| `review_required` | error | commands, push | The change kind is in `review`, or main is protected, and it was about to be pushed straight to main |
| `not_owner` | info | plan | The change touches an area you don't own; it will need that owner's approval |

## Errors and warnings

The rule for severity: **error** when going ahead could lose data or would have to guess (exit 1, nothing changed); **warning** when the tree is safe but someone has to decide (exit 0, listed in `warnings`, an error under `--strict`); **info** for divergence the design expects. All use jd's `--json` shape, so agents can match on `code`. The layout codes are in Config layouts above.

| Code | Level | When | Why |
| --- | --- | --- | --- |
| `duplicate_number` | error | check, plan, apply | Two folders or JDex entries share a number; nothing can be matched |
| `rename_conflict` | error | plan, apply, commands | Config and disk, or two machines, changed one name differently |
| `unknown_baseline` | error | first plan on a machine | No applied record and the names differ; rerun with `--prefer config` or `--prefer disk` |
| `number_conflict` | error | `jd new`, plan | Both sides want one number for different things |
| `number_retired` | error | evaluation | A number that is archived or renumbered away (`from`) is used again |
| `reserved_number` | error | evaluation, `jd new` | Category `x9` or area `90-99` declared for something other than the archive |
| `renumber_not_supported` | error | plan | A removal plus an addition looks like a renumber; add `from` or use `jd renumber` |
| `pending_creates` | error | `jd new` | The category has declared, unapplied entries; jd would take their number |
| `no_plan` | error | apply | No saved plan to run |
| `stale_plan` | error | apply | Config or disk changed since `jd plan` |
| `undo_order` | error | `jd undo` | Something newer has to be undone first |
| `archive_partial` | error | `jd archive`, apply | The folder moved but the JDex entry didn't; names both |
| `jdex_changed` | error | apply | The JDex file changed between read and write (an Emacs buffer saved over it) |
| `config_unwritable` | error | commands | `jdncl` couldn't write the change, so the command didn't run |
| `wp_without_id` | error | evaluation, check, plan | A work package folder or entry without `AC.ID` |
| `category_outside_area` | error | evaluation | A category declared in the wrong area (exists today) |
| `backend_mismatch` | error | check, plan | A folder isn't what its storage says |
| `backend_not_empty` | error | plan | jd won't clone into or mount over a folder with contents, unless the plan migrates them |
| `renamed_on_disk` | warning | check, plan | Central layout: a folder or JDex name changed by hand; the plan offers to record it |
| `deleted_on_disk` | warning | check, plan | An applied entry is gone from disk; the plan offers to recreate or archive it |
| `orphaned` | warning | check, plan | Removed from the config, still on disk |
| `undeclared` | warning | check, plan | On disk, in no config file; the plan offers to record it |
| `strands_moves` | warning | plan, `jd rename` | N journaled moves will no longer be undoable by path |
| `open_work_packages` | warning | plan, `jd archive` | Archiving an ID, category or area whose work packages aren't archived |
| `wp_name_form` | warning | check, plan | A work package folder uses `~` next to `+` folders, or a space without them; the plan fixes it |
| `dangling_ref` | warning | check | A `+ REF` points at a number that doesn't exist |
| `ref_to_archived` | warning | check | A `+ REF` points at an archived number |
| `backend_unavailable` | warning | check, plan, `jd move` | A backend is offline; that folder and everything under it is skipped |
| `app_entry_left` | warning | undo, rename | The task app or note app entry couldn't be changed; prints its link |
| `jdex_name_mismatch` | info | check, plan | Folder and JDex entry disagree; the plan updates the side that didn't change |
| `stale_ref_name` | info | check | A `+ REF` carries an old name for a renamed number |

## Open questions and next steps

Open:

- **Shared systems, the details.** See the section above. Open: whether `jd share sync` should also manage members and teams on the host, or leave that to the host's own settings.
- **Storage kinds and shape.** Is `'dir`, `'path`, `'repo`, `'subvolume`, `'rclone`, `'syncthing`, `'fuse` plus `encrypt` the right set, and is `storage = { kind = … }` on the entry the right shape?
- **The distributed layout across machines.** Is `pulled_rename` acceptable, or should we recommend the central layout for more than one machine?
- **`jdncl` or `jd ncl`.** A tool of its own, or a jd subcommand?

Settled:

- **Applied record:** a jj bookmark per machine, `jd-applied/<host>`; no `applied.json`.
- **No `cli.ncl`:** `jdncl` is the only writer of the config.
- **Disk edits:** `jd plan` reports them and offers a step that records them; `jd apply` runs it.
- **`+ REF` folders** are declarable in both layouts; the contracts must accept `+`.
- **Storage** is per folder, declared by the parent; the config repo travels, the root doesn't. A `storage` set on the system, an area or a category is the default for everything under it, and anything lower can override it. With none set, folders are plain and local, which is right for one machine; adding a second machine is one line and a `jd plan`.
- **Archives:** four strategies, `'standard` (default), `'last_slot`, `'in_place`, `'separate_root`, chosen per system and overridable lower down.

Suggested order of work, each its own PR after #5:

1. Renames (`meta-system.ncl`, `jd.meta-system`, `JD_META_SYSTEM`) and contracts for `+` folders, `from`, `archived`, `storage`, with `number_retired` and `wp_without_id`.
2. Match by number in `jd check`, add `duplicate_number`, drop `jd scaffold`. This fixes the duplicate-folder bug on its own.
3. `jdncl`, central layout first.
4. jj: repo setup, the per-machine bookmark, `jd plan` and `jd apply` with the saved plan and the three-way check.
5. The `jd new` wrapper (`pending_creates`), `jd rename`, `jd archive`, `jd renumber`, with the work package naming rule and `+` folders.
6. `jd undo` over jj and jd's journal, and adapter remove and rename functions (Org and Things first).
7. The distributed layout: `.jd.ncl`, the generated index, `jd convert`, `pulled_rename`.
8. Storage backends: `'dir`, `'path` and `'repo` first, then the mounts with home-manager units.

## Later ideas

Not planned, kept so they don't get lost.

- **A web front end on a Cloudflare Durable Object** (Ash, 2026-11-12). One Durable Object holds a shared system's Org JDex and merges live edits from several people, which fits shared systems. The page parses and edits Org with an existing JS parser (uniorg or orga); Emacs users reach the same file through crdt.el or a sync step. Emacs itself in the browser through container2wasm works as a demo, but it emulates a whole machine: a download of hundreds of MB and slow typing. The jd rules (numbering, plan, the three-way check) would run in the page as wasm, either bash built for WASI running the shell functions, or the core rules rewritten once (Rust, say) and shared by the CLI and the web. Worth starting once plan and apply are stable.
- **Signed history for shared systems**, borrowed from the Open Integrity Project (proposed, not yet agreed). An optional `shared.integrity = true`: the repo's signed first commit is the system's identity; jd generates `allowed_commit_signers` and a policy file from `owners`; commits are signed with SSH keys at push time (jj rewrites commits too often to sign each snapshot); `jd check` verifies the history with plain git and `ssh-keygen`, so approval no longer depends on trusting the git host. Agents sign with their own keys and can't merge. Their forge itself is a Cloudflare-only concept on hold, so we'd borrow the scheme, not the code.
