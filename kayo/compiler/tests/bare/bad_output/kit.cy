# An output takes one `u8` or a `lent` `array[u8]` (owner,
# 2026-10-09); an `i32` is neither, so the target is refused.
module serial { pub.kit fn write [$b: i32] { } }
MAIN { }
