# `copy 'l'` handed to a call is a second, independent value: the callee
# owns it and gives it back, and the caller's own list is untouched. Read as
# the same bytes, the callee freed the caller's list, which was then used
# and freed again. Built at run time so the calls are not worked out ahead.
extern.C fn exit ['code': i32];
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
fn first ['xs': list[i32]] -> i32 { return 'xs'[pos: 1]; }
MAIN {
    let 'k' := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let 'there' := ptr.is_null[getenv[lend 'k']];
    let mut 'x': i32 = 1;
    if 'there' { change 'x' = 2; }
    let 'l': list[i32] = ['x', 3];
    let 'a' := first[copy 'l'];
    let 'b' := first[copy 'l'];
    let 'c' := first[give 'l'];
    no_check my.exit['a' plus 'b' plus 'c'];
}
