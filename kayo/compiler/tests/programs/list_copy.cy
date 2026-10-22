extern.C fn exit ['code': i32];
MAIN {
    let 'a': std::list[i32] = [1, 2, 3];
    let mut 'b': std::list[i32] = copy 'a';
    std::list.append[lend.mut 'b', value: 4];
    change 'b'[pos: 1] = 9;
    let 'la': i32 = std::list.length[lend 'a'];
    let 'lb': i32 = std::list.length[lend 'b'];
    let 'a1': i32 = 'a'[pos: 1];
    let 'b1': i32 = 'b'[pos: 1];
    no_check my.exit['la' times 1 plus 'lb' times 10 plus 'a1' times 100 plus 'b1' times 1000];
}
