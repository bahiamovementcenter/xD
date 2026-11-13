# Work packages (formal 2.4) as a unit that stacks on any system:
#
#   (import "jd.ncl") & (import "work-packages.ncl")
#
# A work package is a second, flat number space, `W` and four digits, linked
# to a number of the system it stacks on: `W0012~11.13 2026 tax return`. It
# takes that number from the system's `pattern`, so it works with JD or with
# any other pattern, and adds nothing to the core's `scheme`.
let xd = import "xd.ncl" in
let { slot, sep, literal, label, .. } = xd in
{
  pattern
    | doc "The number of the system this stacks on, which links point to."
    | not_exported,
  work_packages
    | doc "Work package numbers: W0012, or W0012~11.13 when linked."
    | not_exported
    =
      xd.pattern [
        literal "W",
        slot { name = "work package", width = 4 },
        xd.link { target = pattern },
        sep.label,
        label,
      ],
}
