# The package's entry point: `let xd = import xd in ...` gives xd.ncl's
# primitives and contracts, plus the units that stack with `&`:
# `xd.units.jd & xd.units.work_packages`.
(import "xd.ncl")
& {
  units = {
    jd = import "jd.ncl",
    work_packages = import "work-packages.ncl",
  },
}
