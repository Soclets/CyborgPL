# `mod` on a bin or brain16 takes the divisor's sign (owner, 2026-10-02: a):
# -370 mod 360 is 350, 7.5 mod -2.0 is -0.5, a brain16 -5 mod 3 is 1; a zero
# is 0, never -0 (owner, 2026-10-07: "a"; until then it took the divisor's
# sign, "-0"): 4.0 mod -2.0 is "0"; a tiny negative over 360 that rounds up to
# 360 is 0, always below the divisor (owner: b).
extern.C fn exit ['code': int32];
MAIN {
    let 'angle': bin64 = -370.0;
    let 'turn': bin64 = 360.0;
    let 'a': bin64 = 7.5;
    let 'b': bin64 = -2.0;
    let 'h': brain16 = -5.0;
    let 'k': brain16 = 3.0;
    let 'four': bin64 = 4.0;
    let 'tiny': bin64 = -1.0 div.near (10.0 pow 20.0);
    let 'm1' := to.int32['angle' mod.floor 'turn'] otherwise -1;
    let 'm2' := to.int32[('a' mod.floor 'b') times 10.0] otherwise -1;
    let 'm3' := to.int32['h' mod.floor 'k'] otherwise -1;
    let 'z' := to.stored.str.utf8['four' mod.floor 'b'];
    let 'm4' := to.int32[str.bytes[lend 'z']];
    let 'm5' := to.int32['tiny' mod.floor 'turn'] otherwise -1;
    no_check my.exit[('m1' plus 'm2' plus 'm3' times 7 plus 'm4' times 11 plus 'm5') mod.floor 256];
}
