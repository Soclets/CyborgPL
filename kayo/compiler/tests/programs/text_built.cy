extern.C fn exit ['code': i32];
MAIN {
    let mut 'out': str.utf8 = str.empty;
    for 'i': i32 from 1 to 20 by 1 {
        change 'out' = copy 'out' concat "ab";
    }
    let 'n': i32 = str.bytes[lend 'out'];
    let 'lit': str.utf8 = "fixed";
    let 'dup': str.utf8 = copy 'lit';
    let 'm': i32 = str.bytes[lend 'dup'];
    no_check my.exit['n' plus 'm'];
}
