# What xD's library can express, and who needs what it can't (ADR 0007).
#
# Each capability has a code. When a configuration asks for one that isn't
# built, xd.ncl fails with "capability '<code> isn't built yet", and an agent
# looks the code up here and follows AGENTS.md, "Expanding the library".
#
# - `configs`: Nickel files under nickel/ that use the capability. A system
#   counts as a consumer only once it is written as one of these: the survey
#   alone doesn't count (Ash, 2026-11-13).
# - `known`: systems from the survey (project files,
#   research/addressable-systems.md) that would need it. Candidates for the
#   second config, nothing more.
#
# crates/xd-nickel/tests/capabilities.rs enforces the rules:
#
# - 'built: every config evaluates, and there are at least two of them,
#   unless `grandfathered` (built before this rule existed).
# - 'proposed and 'deferred: every config fails with this capability's code,
#   and there is at most one. A second config means the capability is ready
#   to build, and the test says so until it is.
# - Every code xd.ncl reports is listed here, and every file in examples/ is
#   some capability's config.
let Capability = {
  summary | String,
  status | [| 'built, 'proposed, 'deferred |],
  adr
    | String
    | doc "The ADR that decides it, by number."
    | optional,
  grandfathered
    | Bool
    | doc "Built before ADR 0007, so exempt from the two-config rule."
    | default
    = false,
  configs | Array String | default = [],
  known | Array String | default = [],
}
in
{
  capabilities
    | { _ | Capability }
    = {
      # Built
      fixed_slots = {
        summary = "Slots of a fixed number of digits from one alphabet.",
        status = 'built,
        adr = "0004",
        configs = ["jd.ncl", "work-packages.ncl"],
      },
      separators = {
        summary = "The separators of formal 3.1 and literal text between slots.",
        status = 'built,
        adr = "0004",
        configs = ["jd.ncl", "work-packages.ncl"],
      },
      labels = {
        summary = "Free text after the number, which a rename changes.",
        status = 'built,
        adr = "0004",
        configs = ["jd.ncl", "work-packages.ncl"],
      },
      ranges = {
        summary = "A level written as the span of its children, as JD's 10-19.",
        status = 'built,
        adr = "0004",
        grandfathered = true,
        configs = ["jd.ncl"],
      },
      links = {
        summary = "A number linking to another pattern's number, as W0012~11.13.",
        status = 'built,
        adr = "0004",
        grandfathered = true,
        configs = ["tests/primitives.ncl"],
      },
      stacking = {
        summary = "Units that stack with &, each usable alone.",
        status = 'built,
        adr = "0004",
        grandfathered = true,
        configs = ["tests/primitives.ncl"],
      },
      per_slot_alphabets = {
        summary = "Each slot with its own alphabet, as MSC's 68T05.",
        status = 'built,
        adr = "0004",
        configs = ["examples/msc.ncl", "examples/acm-ccs-1998.ncl"],
        known = ["IPC", "Maidenhead", "NATO Stock Number"],
      },

      # Designed in an ADR, not built
      facets = {
        summary = "Named fields on entries from a code list, inherited down the tree.",
        status = 'proposed,
        adr = "0005",
        known = ["PARA", "JD archive strategies"],
      },
      lifecycles = {
        summary = "A state facet with allowed transitions and effects such as 'retire.",
        status = 'proposed,
        adr = "0005",
        known = ["PARA", "JD archiving"],
      },
      placement = {
        summary = "An entry's root folder and layout, chosen from its facets.",
        status = 'proposed,
        adr = "0005",
        known = ["PARA", "JD archive strategies"],
      },
      entry_form = {
        summary = "Entries as folders, files or either; lone files wrapped in a folder.",
        status = 'proposed,
        adr = "0005",
        known = ["PARA", "JD"],
      },
      hidden_identity = {
        summary = "An identity kept out of the name, generated and stored beside the entry.",
        status = 'proposed,
        adr = "0006",
        known = ["PARA alone", "JD aliases"],
      },

      # From the survey, section 4: not designed yet
      variable_width = {
        summary = "Slots of a minimum to maximum number of digits, or unbounded.",
        status = 'deferred,
        configs = ["examples/dewey.ncl"],
        known = ["UDC", "LCC", "Colon", "Luhmann", "ACM CCS 2012", "IPC", "IPv4", "IPv6", "DNS", "Legal outlines", "DOI", "SemVer"],
      },
      slot_order = {
        summary = "How a slot sorts ('lexical, 'numeric, 'fractional) and value bounds.",
        status = 'deferred,
        known = ["Dewey", "UDC", "LCC", "Luhmann", "IPC", "IPv4", "Legal outlines", "SemVer"],
      },
      repeat = {
        summary = "A slot or sub-pattern repeated, to unbounded depth.",
        status = 'deferred,
        known = ["Dewey", "UDC", "LCC", "Colon", "Luhmann", "DNS", "Legal outlines", "DOI", "Geohash", "Maidenhead", "SemVer"],
      },
      cycle = {
        summary = "Levels that alternate alphabets, as Luhmann's 1a2b.",
        status = 'deferred,
        known = ["Luhmann", "Maidenhead"],
      },
      optional_parts = {
        summary = "Parts of a pattern that may be absent.",
        status = 'deferred,
        known = ["Harmonized System", "Dewey", "UDC", "LCC", "Colon", "ISO 8601", "NAICS", "ICD-10-CM", "SKU", "SemVer", "File plans"],
      },
      choice = {
        summary = "One of several sub-patterns.",
        status = 'deferred,
        known = ["UDC", "ISO 8601", "ISBN", "IBAN", "DOI", "URN"],
      },
      code_lists = {
        summary = "A slot whose values come from a table of codes of any width.",
        status = 'deferred,
        known = ["UDC", "Colon", "PARA", "ICD-10-CM", "SKU", "IBAN", "URN", "File plans"],
      },
      enclose = {
        summary = "A part wrapped in paired delimiters, as UDC's (410).",
        status = 'deferred,
        known = ["UDC", "Legal outlines"],
      },
      check_digits = {
        summary = "A slot computed from the others: Luhn, mod 10, mod 11, mod 97.",
        status = 'deferred,
        known = ["ISBN", "GTIN", "IBAN", "SKU"],
      },
      cross_slot_checks = {
        summary = "Validity that depends on several slots, as a day valid for its month.",
        status = 'deferred,
        known = ["ISO 8601", "File plans"],
      },
      normalization = {
        summary = "Accepted forms beside the canonical one: case, grouping, elision.",
        status = 'deferred,
        known = ["NATO Stock Number", "Dewey", "ISO 8601", "ISBN", "GTIN", "IBAN", "IPv6", "DNS", "ULID", "Maidenhead"],
      },
      numeral_kinds = {
        summary = "Slots that aren't positional with a zero: bijective base-26, roman numerals.",
        status = 'deferred,
        known = ["Legal outlines", "Luhmann"],
      },
      wildcards = {
        summary = "Placeholders and ranges over any slot, as MSC's 68Txx or NAICS's 31-33.",
        status = 'deferred,
        known = ["MSC", "NAICS", "UDC"],
      },
      explicit_hierarchy = {
        summary = "A parent given by data, not computed from the digits.",
        status = 'deferred,
        known = ["LCC", "IPC", "ACM CCS 2012"],
      },
      reversed_significance = {
        summary = "The most significant level written last.",
        status = 'deferred,
        known = ["DNS"],
      },
      dependent_width = {
        summary = "A slot's width or codes set by an earlier slot's value.",
        status = 'deferred,
        known = ["ISBN", "GTIN", "IBAN", "UDC special auxiliaries"],
      },

      # In Nickel, not yet in the Rust core (ADR 0004, decision 3)
      core_mixed_alphabets = {
        summary = "The core holding a different alphabet per slot.",
        status = 'deferred,
        known = ["MSC", "ACM CCS 1998", "Maidenhead"],
      },
      core_mixed_range_separators = {
        summary = "The core holding a different range separator per slot.",
        status = 'deferred,
      },
      core_links = {
        summary = "The core holding links between number spaces.",
        status = 'deferred,
        known = ["Work packages on JD"],
      },
      core_trailing_text = {
        summary = "The core holding text after the number other than a space and a label.",
        status = 'deferred,
        known = ["JD + children"],
      },
    },
}
