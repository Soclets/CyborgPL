# A count of bare numbers only is worked out exactly (operators.md): the
# product in between, -2^125, fits no type, and mod 9 it is 4, so four
# items (check claude, seed 11045; the oracle agrees).
extern.C fn exit ['code': int32];
MAIN {
    let 'xs' := list.fill[value -336, count ((-9223372036854775808) times 4611686018427387904) mod.floor 9];
    no_check my.exit[to.int32[list.length[lend 'xs']]];
}
