# xD

A Johnny.Decimal system where a Nickel config and the `jd` CLI share one tree:
either side can create, rename, archive or renumber, `jd plan` / `jd apply`
bring the disk in line, and jj (Jujutsu) keeps the history.

Status: design only. Nothing is implemented yet.

- Design: [docs/design/workflow-rules.md](docs/design/workflow-rules.md),
  exported from the live doc at
  https://claude.ai/artifact/WAJtDtQhqdfsYjc83Ga5wv (edit there, then re-export).
- Related: [bahiamovementcenter/johninunix](https://github.com/bahiamovementcenter/johninunix),
  the Nix flake around the upstream jd CLI that this builds on.

## Development

```sh
nix develop    # jj, nickel, nushell, jq
```
