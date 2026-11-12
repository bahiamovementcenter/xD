# xD

Extensible Decimal: a library of composable primitives to construct organizational systems.

Johnny.Decimal is one configuration built from it, and the default. How xD is built is
recorded in [docs/adr/0001-architecture.md](docs/adr/0001-architecture.md); the primitives
are described in [docs/jd-formal.md](docs/jd-formal.md) and
[docs/jd-experiments.md](docs/jd-experiments.md).

## Crates

- [`xd-core`](crates/xd-core): addresses and their hierarchy, entries, allocation and
  retirement, and folder names, generic over a numbering scheme (base and levels).
  Johnny.Decimal's scheme and standard zeros are in `xd_core::jd`.

## Development

```sh
nix develop        # or direnv allow
cargo test
```
