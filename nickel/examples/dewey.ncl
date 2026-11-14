# Dewey Decimal Classification: three digits (class, division, section),
# a point, then any number of further digits, as in 516.375. Its first
# consumer of 'variable_width (capabilities.ncl), so this fails until that
# capability is built; crates/xd-nickel/tests/capabilities.rs checks that it
# fails with that code. Dewey also sorts the digits after the point as a
# decimal fraction ('slot_order), not used here yet.
let xd = import "xd.ncl" in
let { slot, sep, label, .. } = xd in
let dewey =
  xd.pattern [
    slot { name = "class", width = 1 },
    slot { name = "division", width = 1 },
    slot { name = "section", width = 1 },
    sep.within,
    slot { name = "subdivision", width = { min = 1 } },
    sep.label,
    label,
  ]
in
[
  dewey.parse "516.375 Algebraic geometry" == { address = [5, 1, 6, 375], label = "Algebraic geometry" },
]
|> std.test.assert_all
