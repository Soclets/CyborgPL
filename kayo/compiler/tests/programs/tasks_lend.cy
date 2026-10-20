extern.C fn exit ['code': i32];
fn work ['n': i32] -> i32 { return 'n' times 2; }
fn fill [lent.mut 'into': list[i32], 'from': i32] {
    for 'i': i32 from 1 to 5 by 1 {
        list.append[lend.mut 'into', value 'from' plus 'i'];
    }
}
MAIN {
    let mut 'a': list[i32] = [];
    let mut 'b': list[i32] = [];
    {
        concur with [lend.mut 'a'] { fill[lend.mut 'a', 10]; }
        concur with [lend.mut 'b'] { fill[lend.mut 'b', 20]; }
        concur work[4];
    }
    let 'la': i32 = list.length[lend 'a'];
    let 'lb': i32 = list.length[lend 'b'];
    let 'last': i32 = 'b'[pos 5];
    no_check my.exit['la' plus 'lb' plus 'last'];
}
