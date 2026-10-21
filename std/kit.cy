# std, CyborgPL's standard library, written in CyborgPL (owner, 2026-09-14; again 2026-10-14, "(c)").
# Every public item may be written with `std::` or without it (owner, 2026-10-06: "optional for all").
bare.kit_prefix.optional [fn, op, struct, enum, interface];
# and every enum's values may drop the enum's name where the type is known (owner, 2026-10-20: "optional for all of `std`")
bare.enum_prefix.optional [enum];

# Whose case rules a full case change follows (owner, 2026-10-20: "casing, go"): Turkish, Azerbaijani and
# Lithuanian have rules of their own in Unicode's tables; `root` is every other language.
pub enum casing { tr, az, lt, root }
