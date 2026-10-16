# A bare whole number given more than int64 holds (2026-10-10): `:=` lets
# Precog choose its width, and it was always 64 bits, so 10^19 wrapped to
# -8446744073709551616 with no word said. (10^19 - 1) div 10^17 is 99.
extern.C fn exit ['code': int32];
MAIN {
    let 'a' := 10000000000000000000;
    let 'b' := 'a' minus 1;
    no_check my.exit[to.int32['b' div.floor 100000000000000000]];
}
