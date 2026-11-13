# Johnny.Decimal as an xD configuration: the defaults. Matches xd_core::jd.
let xd = import "xd.ncl" in
{
  scheme = {
    base = 10,
    levels = [
      { name = "area", width = 1, range = true },
      { name = "category", width = 1 },
      { name = "id", width = 2, separator = "." },
    ],
  },
} | xd.System
