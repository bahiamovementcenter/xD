# xD

Extensible Decimal: a library of composable primitives to construct organizational systems.

Johnny.Decimal is one configuration built from it, and the default. How xD is built is
recorded in [docs/adr/0001-architecture.md](docs/adr/0001-architecture.md); the primitives
are described in [docs/jd-formal.md](docs/jd-formal.md) and
[docs/jd-experiments.md](docs/jd-experiments.md). How it's meant to be extended, and what
is deliberately fixed, is in [docs/extensibility.md](docs/extensibility.md); the patterns
the design builds on are in [docs/design-patterns.md](docs/design-patterns.md).

## Crates

- [`xd-core`](crates/xd-core): addresses and their hierarchy, entries, allocation and
  retirement, and folder names, generic over a numbering scheme (base and levels).
  Johnny.Decimal's scheme and standard zeros are in `xd_core::jd`.
- [`xd-nickel`](crates/xd-nickel): loads a system's configuration from Nickel through
  [`nickel-embed`](https://github.com/bahiamovementcenter/rust-nickel).

## Nickel

- [`nickel/xd.ncl`](nickel/xd.ncl): xD's primitives (alphabets, slots, separators, links,
  labels, patterns; [ADR 0004](docs/adr/0004-nickel-primitives.md)), built on Nickel's
  std, and the contracts a configuration is checked against. Any program that embeds
  Nickel can use them: `(import "jd.ncl").pattern.parse "11.11 Passport"`.
- [`nickel/jd.ncl`](nickel/jd.ncl): Johnny.Decimal, the default configuration. Build on it
  with a merge, e.g. `(import "jd.ncl") & { scheme.base | force = 16 }`.
- [`nickel/work-packages.ncl`](nickel/work-packages.ncl): work packages as a unit that
  stacks on any system: `(import "jd.ncl") & (import "work-packages.ncl")`.

## Development

```sh
nix develop        # or direnv allow
cargo test
```
