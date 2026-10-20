# An iterator handing out owned lists (owner, 2026-10-09): each item is
# freed after its trip, and `perm` keeps the last one past the loop.
extern.C fn exit ['code': i32];

struct Rows {
    left: i32
}

fn Rows.next [lent.mut 'r': Rows] -> optional[list[i32]] {
    if 'r'.left equal_to 0 { return optional.none; }
    change 'r'.left = 'r'.left minus 1;
    let mut 'row': list[i32] = [1, 2];
    list.append[lend.mut 'row', value 'r'.left];
    return optional.value[give 'row'];
}

MAIN {
    let mut 'total': i32 = 0;
    let mut 'r' := Rows[left = 4];
    for 'row' in lend.mut 'r' {
        change 'total' = 'total' plus to.i32[list.length[lend 'row']];
    }
    let 's' := Rows[left = 3];
    for perm 'last' in give 's' {
        change 'total' = 'total' plus 1;
    }
    no_check my.exit['total' plus to.i32[list.length[lend 'last']]];
}
