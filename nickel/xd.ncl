# The contracts every xD configuration is checked against. They mirror
# xd_core::Scheme, so a configuration that passes them deserializes into it;
# xd_core::Scheme::validate still checks what a contract can't see as
# easily (digits that clash with separators, capacity overflow).
let NonEmptyString = std.contract.from_predicate (fun s => std.is_string s && s != "") in
{
  Level = {
    name | NonEmptyString | doc "What this level is called, e.g. \"area\".",
    width
      | std.number.PosNat
      | doc "Digits this level adds to the number.",
    separator
      | String
      | doc "Written before this level's digits: \"\" for JD's category, \".\" for its ID."
      | default
      = "",
    range
      | Bool
      | doc "Written as the span of its children, as JD writes area 1 as 10-19."
      | default
      = false,
  },

  Scheme = {
    base
      | std.number.Nat
      | std.contract.from_predicate (fun b => b >= 2)
      | doc "How many digits there are.",
    alphabet
      | String
      | doc "The digits, lowest first, exactly `base` of them. Without one, digits are 0-9A-Z (case-insensitive) and the base is at most 36."
      | optional,
    range_separator
      | NonEmptyString
      | doc "Between the first and last child of a level written as a range."
      | default
      = "-",
    levels
      | Array Level
      | std.contract.from_predicate (fun ls => std.array.length ls > 0)
      | doc "From the top of the tree down.",
  },

  System = {
    scheme | Scheme,
  },
}
