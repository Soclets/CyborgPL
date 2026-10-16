extern.C fn exit ['code': int32];
MAIN {
    let 'xs': list[int32] = [4, 5, 6];
    let mut 'acc': int32 = 0;
    for 'x' in lend 'xs' {
        change 'acc' = 'acc' plus 'x';
    }
    no_check my.exit['acc'];
}
