# 7. A capability registry, and when an agent expands the library

Status: proposed, 2026-11-13.

## Context

[extensibility.md](../extensibility.md) builds a mechanism only when a
second real use appears. Until now that was a judgement made in review. Ash
asked (2026-11-13) for it to be mechanical: an agent working with xD that
finds the library can't express a system design should be able to tell
whether another known system needs the same thing, and if so, take the steps
to build it.

Two things made that possible. ADR 0004 put xD's primitives in Nickel, where
a configuration fails with a message at evaluation. And the survey of other
addressing systems (project files, `research/addressable-systems.md`)
listed, for each missing capability, the systems that need it.

Ash also decided what counts as a use: **a system counts only once it is
written as a config**. A name in the survey is a candidate, not a consumer.

## Decisions

### 1. Capabilities have codes, and the library reports them

Every planned feature the library lacks has a code (`variable_width`,
`repeat`, `facets` …). When a configuration asks for one, `xd.ncl` fails with

```
capability 'variable_width isn't built yet: a slot's width is a fixed number of digits, for now
```

and a note pointing to the registry and to AGENTS.md. Configurations ask
for an unbuilt feature by writing it the way they would once it exists:

| Written as | Reports |
| --- | --- |
| a slot's `width = { min, max }` | `variable_width` |
| a slot's `order`, `min`, `max` | `slot_order` |
| a slot's `numeral` | `numeral_kinds` |
| a pattern part of kind `'repeat`, `'cycle`, `'optional`, `'choice`, `'enclose`, `'codes`, `'check` | `repeat`, `cycle`, `optional_parts`, `choice`, `enclose`, `code_lists`, `check_digits` |
| a system's `facets`, `lifecycle`, `placement`, `entry_form`, `identity` | the capabilities of ADRs 0005 and 0006 |
| a pattern the core can't hold, through `scheme` | `core_mixed_alphabets`, `core_links` … |

The system fields matter most: `System` is open (`..`), so without this, a
`facets` field written before facets exist would be silently ignored.

### 2. `nickel/capabilities.ncl` is the registry

Each capability has a `summary`, a `status` (`'built`, `'proposed` with an
ADR, or `'deferred`), its `adr`, its `configs` (Nickel files under
`nickel/`, usually `nickel/examples/`), and its `known` systems from the
survey. It is seeded with the survey's section 4 and 5 capabilities, the
proposals of ADRs 0005 and 0006, and what ADR 0004 left for the core.

### 3. A test enforces the rules

`crates/xd-nickel/tests/capabilities.rs`:

- A **built** capability has at least two configs, and each evaluates (its
  `std.test` assertions pass). Capabilities built before this record are
  `grandfathered` and need only their existing configs to keep working.
- An **unbuilt** capability's configs each fail with its code, and there is
  at most one. **A second config fails the test with "it's time to build
  it"**: that is the trigger.
- Every code `xd.ncl` reports is in the registry, and every file in
  `nickel/examples/` is some capability's config.

### 4. The procedure

AGENTS.md ("Expanding the library") and the repository skill
`.claude/skills/expand-library/SKILL.md` give the steps: look the code up,
write your system as an example config, stop at one config, and at two
write or extend the ADR, build the cheapest mechanism that serves both,
mark it built and make both configs pass. One capability per pull request.

### 5. Limits

- An agent may write the second config itself from a `known` system, from
  that system's source. It may not lower the two-config rule, mark a
  capability built without both configs passing, or add `grandfathered`.
- A capability whose ADR would change a sealed rule (extensibility.md) stops
  at the ADR for the owner's approval.
- Agents never merge.

### 6. First entries

`per_slot_alphabets`, built in ADR 0004, is the first capability to meet the
rule properly: `examples/msc.ncl` (MSC2020, `68T05`) and
`examples/acm-ccs-1998.ncl` (`D.3.2`) are its two configs. `variable_width`
has its first: `examples/dewey.ncl` (`516.375`), which fails with
`capability 'variable_width` and waits for a second.

## Consequences

- "Two real uses" becomes something CI checks rather than something review
  remembers.
- The survey's systems turn into executable examples one at a time, as
  something needs them, each with round-trip assertions that become tests
  the moment its capability is built.
- An unbuilt feature fails loudly instead of being ignored.
- Adding a capability code to `xd.ncl` without registering it fails CI, so
  the registry can't drift from the library.
- rust-nickel vendors `xd.ncl`; this changes its error messages and slot
  contract but not its file layout or field names.

## Open

- Whether the registry should also record the core's capabilities (the Rust
  side), or only the Nickel library's.
- Whether an agent outside this repository (one using xD as a library)
  should file its config as an issue or a pull request.
