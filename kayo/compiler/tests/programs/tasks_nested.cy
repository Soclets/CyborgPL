extern.C fn exit ['code': i32];
fn total [lent 'xs': list[i32]] -> i32 {
    let mut 's': i32 = 0;
    for 'x' in lend 'xs' { change 's' = 's' plus 'x'; }
    return 's';
}
MAIN {
    let 'xs': list[i32] = [1, 2, 3];
    let 'name': str.utf8 = "task" concat "s";
    let mut 'out': list[i32] = [];
    for 'k': i32 [from: 1, to: 50, by: 1] {
        concur with [copy 'xs', give 'name', lend.mut 'out'] {
            let 'inner' := total[lend 'xs'];
            {
                concur with [lend 'xs'] { let 'again' := total[lend 'xs']; }
            }
            list.append[lend.mut 'out', value: 'inner' plus str.bytes[lend 'name']];
        }
        break;
    }
    no_check my.exit[list.length[lend 'out'] plus 'out'[pos: 1]];
}
