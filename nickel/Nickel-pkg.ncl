# xD's primitives as a Nickel package, so programs that embed Nickel can pin
# them (ADR 0004): `import xd` loads main.ncl.
{
  name = "xd",
  version = "0.1.0",
  minimal_nickel_version = "1.18.0",
  authors = ["Ash"],
  description = "xD's primitives: alphabets, slots, separators, links, labels and patterns, plus the JD and work-package units.",
  dependencies = {},
} | std.package.Manifest
