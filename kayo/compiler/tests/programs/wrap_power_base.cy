# `xx~` wraps at the base's size alone: the exponent is a count, like a
# shift's, so a u32 base with an i32 exponent is not mixing sizes
# (operators.md, the oracle's reading). The exponent is known only at run
# time: whether an unset variable is there.
extern.C fn exit ['code': i32];
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
MAIN {
    let 'k' := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let 'there' := ptr.is_null[getenv[lend 'k']];
    let mut 'n': i32 = 26;
    if 'there' { change 'n' = 25; }
    let 'h': u32 = 3;
    let 'p' := 'h' pow.wrap 'n';
    let 'q': u8 = 7;
    let 'r' := 'q' pow.wrap 'n';
    no_check my.exit[to.i32[('p' mod.floor 256) plus ('r' mod.floor 8)]];
}
