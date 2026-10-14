extern.C fn exit ['code': int32];
MAIN {
    let mut 'out': str.utf8 = str.empty;
    for 'i': int32 from 1 to 20 by 1 {
        change 'out' = copy 'out' ++ "ab";
    }
    let 'n': int32 = str.bytes[lend 'out'];
    let 'lit': str.utf8 = "fixed";
    let 'dup': str.utf8 = copy 'lit';
    let 'm': int32 = str.bytes[lend 'dup'];
    no_check my.exit['n' plus 'm'];
}
