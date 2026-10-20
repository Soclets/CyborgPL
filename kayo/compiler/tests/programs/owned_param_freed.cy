# An owned parameter is the function's to give back when it ends: nothing
# freed a list handed over with `give`, and every such call leaked. The
# list is built at run time so the call is not worked out in advance.
extern.C fn exit ['code': i32];
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
fn total ['xs': list[i32]] -> i32 { return 'xs'[pos: 1]; }
MAIN {
    let 'k' := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let 'there' := ptr.is_null[getenv[lend 'k']];
    let mut 'x': i32 = 1;
    if 'there' { change 'x' = 2; }
    let 'l': list[i32] = ['x', 3];
    let 'a' := total[give 'l'];
    no_check my.exit['a' minus 2];
}
