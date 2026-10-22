# std, CyborgPL's standard library, written in CyborgPL (owner, 2026-09-14; again 2026-10-14, "(c)").
# Other kits write std's names with their path, `std::str.split[…]`, or bring one in with `use std::str as str;`
# (owner, 2026-10-22); inside std they are its own.

# Whose case rules a full case change follows (owner, 2026-10-20: "casing, go"): Turkish, Azerbaijani and
# Lithuanian have rules of their own in Unicode's tables; `root` is every other language.
pub enum casing { tr, az, lt, root }
