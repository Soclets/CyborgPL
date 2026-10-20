# A `:=` list of bare numbers takes its width from its use (types.md:
# Precog picks the type): lent to a `list[i32]`, it is one. Left at the
# exact width, the callee read the wrong layout and summed garbage.
extern.C fn exit ['code': i32];
fn total [lent 'xs': list[i32]] -> i32 {
    let mut 'n': i32 = 0;
    for 'x' in lend 'xs' { change 'n' = 'n' plus 'x'; }
    return 'n';
}
MAIN {
    let 'scores' := [72, 40, 95, 55];
    let 'first' := 'scores'[pos 1] plus 1;
    no_check my.exit[(total[lend 'scores'] plus to.i32['first']) mod.floor 256];
}
