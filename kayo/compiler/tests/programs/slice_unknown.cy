# A slice whose end is not known is checked at run time: past the end
# of the list, it stops. `strlen` is extern, so 'k' is any `u64`.
extern.C fn exit ['code': i32];
extern.C fn strlen ['s': ptr[u8]] -> u64;
MAIN {
    let 's' := "nine char";
    no_check let 'k': u64 = strlen[lend 's'];
    let 'xs': list[i64] = [1, 2, 3];
    let 'part' := list.slice[lend 'xs', from pos 1, to pos 'k'];
    no_check my.exit[to.i32[list.length[lend 'part']]];
}
