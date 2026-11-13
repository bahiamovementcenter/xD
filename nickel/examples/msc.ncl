# The Mathematics Subject Classification (MSC2020) as a pattern: two digits,
# a capital letter, two digits, as in 68T05 (artificial intelligence,
# learning and adaptive systems). A consumer of per-slot alphabets
# (capabilities.ncl). Coarser levels are written as plain prefixes here
# (68, 68T); MSC's own 68-XX and 68Txx placeholders need 'wildcards.
let xd = import "xd.ncl" in
let { slot, sep, label, alphabets, .. } = xd in
let msc =
  xd.pattern [
    slot { name = "field", width = 2 },
    slot { name = "subfield", width = 1, alphabet = alphabets.of "ABCDEFGHIJKLMNOPQRSTUVWXYZ" },
    slot { name = "topic", width = 2 },
    sep.label,
    label,
  ]
in
[
  msc.render [68, 19, 5] == "68T05",
  msc.parse "68T05 Learning and adaptive systems" == { address = [68, 19, 5], label = "Learning and adaptive systems" },
  msc.parse "68T" == { address = [68, 19] },
  msc.parse "68t05" == null,
  msc.write (msc.parse "53A45 Differential geometry") == "53A45 Differential geometry",
]
|> std.test.assert_all
