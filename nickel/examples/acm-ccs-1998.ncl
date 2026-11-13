# The 1998 ACM Computing Classification System as a pattern: a letter, then
# two levels of one character, as in D.3.2 (programming languages, language
# classifications), where `m` is the "miscellaneous" slot after 9. A
# consumer of per-slot alphabets (capabilities.ncl). Illustrative: the
# survey couldn't verify the 1998 code format against a source.
let xd = import "xd.ncl" in
let { slot, sep, label, alphabets, .. } = xd in
let level = alphabets.of "0123456789m" in
let ccs =
  xd.pattern [
    slot { name = "top", width = 1, alphabet = alphabets.of "ABCDEFGHIJK" },
    sep.within,
    slot { name = "second", width = 1, alphabet = level },
    sep.within,
    slot { name = "third", width = 1, alphabet = level },
    sep.label,
    label,
  ]
in
[
  ccs.render [3, 3, 2] == "D.3.2",
  ccs.parse "D.3.2 Language classifications" == { address = [3, 3, 2], label = "Language classifications" },
  ccs.parse "D.3.m" == { address = [3, 3, 10] },
  ccs.parse "D.3" == { address = [3, 3] },
  ccs.parse "L.1.1" == null,
]
|> std.test.assert_all
