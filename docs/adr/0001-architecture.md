# 1. A new Rust tool with a plan-then-apply core

Status: accepted, 2026-11-12.

## Context

xD (Extensible Decimal) is a library of composable primitives to build
organizational systems. Johnny.Decimal (JD) is one configuration built from
it, not the system itself.

The previous attempt, Johninunix, wrapped the upstream `jd` CLI with Nix,
Nickel and Nushell. Most of its problems came from wrapping a tool it didn't
own: jd matches folders by full name, so a rename in Nickel creates a
duplicate; the third work package in a new system collides with `W0003`;
JDex storage is fixed to Markdown or Bear; and behaviour had to be bolted on
from outside (`jd.nu` post-steps, `jdncl` as the only config writer, a
separate check pass). Johninunix is retired.

The primitives are written down in [the formal model](../jd-formal.md) (a
core plus independent modules) and [the experiments](../jd-experiments.md).

## Decisions

1. **A fresh implementation in Rust.** It embeds Nickel through
   [`nickel-embed`](https://github.com/bahiamovementcenter/rust-nickel), so
   no FFI layer sits between the tool and its configuration language. Rust
   also has `jj-lib` for history, compiles to wasm for a later web front
   end, and has the strongest libraries for content digests and Gordian
   Envelopes (experiments E4, E10).
2. **Everything is configurable; the defaults follow the JD spec.** The
   core is generic over the base and the levels of the address (formal 1.1,
   experiment E1), so JD is `base = 10` with an area, a category and a
   two-digit ID. Compliance is with the Johnny.Decimal **spec**
   (johnnydecimal.com), not with the `jd` CLI: xD doesn't read or write jd's
   `config.json` or journal and doesn't copy its commands.
3. **Plan, then apply.** Each command reads the configuration, the disk and
   the JDex into one model; a pure function turns the difference into a
   plan; small backends carry the plan out. The core stays pure and is
   tested without touching a file system.
4. **Seams are traits.** Each row of formal section 4 (numbering scheme,
   reserved slots, allocation policy, projections, archive target, storage,
   app adapters, history) is a Rust trait with plain, serializable input
   and output. JD's modules are built-in implementations, switched on from
   Nickel.
5. **Declarative extension is Nickel; executable extension is wasm
   components.** A system is a Nickel configuration validated by contracts.
   Code that Nickel can't express is a plugin built on the WebAssembly
   Component Model with WIT interfaces (wasmtime `bindgen!`), so a plugin
   whose types don't match is rejected when it loads. Nickel contracts check
   what types can't (a valid number for this system, no reused numbers).
   Extism was considered and rejected: its calls are bytes in, bytes out,
   and nothing checks types at the boundary.
6. **Checks port jdlint.** `xd check` reimplements
   [jdlint](https://github.com/SiriusStarr/jdlint)'s rules over the same
   model, keeping its finding codes, so the xjd Claude Code mod can switch
   from jdlint to `xd check --json`. jdlint is the test oracle until the
   port is complete.
7. **Shared systems are a later backend.** A shared registry (allocation,
   the JDex, owners) sits behind a trait. SpacetimeDB is the first candidate
   to prototype there: an allocation reducer runs as one serialized
   transaction, so collaborators can't take the same number. It isn't used
   for the core: it runs as a server, its modules aren't a cross-language
   plugin system, and it's under the Business Source License until
   2031-11-10.

## Libraries

`nickel-embed`, `serde`, `clap`, `miette` (diagnostics), `jj-lib`, `notify`
(file watching), `orgize` (Org), `comrak` (Markdown), `winnow` (the shorthand
grammar), `blake3` (digests), `wasmtime` (plugins), `nu-plugin` (Nushell
front end).

## Order of work

1. This record, and the formal model and experiments in `docs/`.
2. `xd-core`: address, hierarchy, entries, allocation and projections,
   generic over base and levels.
3. The JD defaults as a Nickel configuration, loaded through
   `nickel-embed`.
4. `xd check`, porting jdlint's rules and comparing with jdlint on fixture
   trees.
5. The xjd mod calls `xd check --json`.

## Consequences

- Rust is slower to iterate in than Nushell or Go, and `nickel-lang-core`
  changes its API often, so `nickel-embed` is maintained alongside xD.
- Nothing from Johninunix is reused as code; its design notes are inputs.
