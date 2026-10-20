extern.C fn exit ['code': i32];
MAIN {
    let 'a': list[i32] = [1, 2, 3];
    let mut 'b': list[i32] = copy 'a';
    list.append[lend.mut 'b', value: 4];
    change 'b'[pos: 1] = 9;
    let 'la': i32 = list.length[lend 'a'];
    let 'lb': i32 = list.length[lend 'b'];
    let 'a1': i32 = 'a'[pos: 1];
    let 'b1': i32 = 'b'[pos: 1];
    no_check my.exit['la' times 1 plus 'lb' times 10 plus 'a1' times 100 plus 'b1' times 1000];
}
