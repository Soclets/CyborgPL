# std, CyborgPL's standard library, written in CyborgPL (owner, 2026-09-14; again 2026-10-14, "(c)").
# Every public item may be written with `std::` or without it (owner, 2026-10-06: "optional for all").
bare.kit_prefix.optional [fn, op, struct, enum, interface];
# and every enum's values may drop the enum's name where the type is known (owner, 2026-10-20: "optional for all of `std`")
bare.enum_prefix.optional [enum];
