extern.C fn exit ['code': i32];
fn fill [lent.mut 'into': list[i32]] {
    list.append[lend.mut 'into', value 7];
}
MAIN {
    let mut 'b': list[i32] = [];
    fill[lend.mut 'b'];
    let 'first': i32 = 'b'[pos 1];
    no_check my.exit['first'];
}
