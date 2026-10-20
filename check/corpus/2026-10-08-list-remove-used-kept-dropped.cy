# Coverage 2026-10-08 (owner: "yea, ask it to cover them"): list.remove with pos: (f2f3879b), its item
# printed straight from the call, kept in a name, and dropped; then the list's length and items.
# check claude generator, seed 1
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': list[u8] = [];
    if 'neg' { list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'ds': list[u8] = [];
    list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': u64 = list.length[lend 'ds'];
    while 'k' at_least 1 {
        list.append[lend.mut 'buf', value: 'ds'[pos: 'k']];
        change 'k' = 'k' minus 1;
    }
    list.append[lend.mut 'buf', value: 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', list.length[lend 'buf']];
}

fn put_u ['n': u64] {
    emit['n', false];
}

fn put_i ['n': i64] {
    if 'n' at_least 0 {
        emit[to.u64['n'], false];
    } else {
        let 'up': i64 = -('n' plus 1);
        emit[to.u64['up'] plus 1, true];
    }
}

fn b1 [@y, @x] [lent @y 'a': list[u16], lent @y 'b': list[u16], lent @x 'c': list[u16]] -> lent @x list[u16] {
    return lend 'c';
}

fn b2 [@x, @y] [lent 'a': list[u16], lent @x 'b': list[u16], lent @y 'c': list[u16]] -> lent @x list[u16] {
    return b1[lend 'b', lend 'a', lend 'b'];
}

fn b3 [@y, @x] [lent @y 'a': list[u16], lent @x 'b': list[u16], lent @x 'c': list[u16]] -> lent @x list[u16] {
    if list.length[lend 'c'] at_least list.length[lend 'a'] {
        return b2[lend 'a', lend 'c', lend 'c'];
    }
    return b1[lend 'c', lend 'c', lend 'b'];
}

fn b4 [@y, @x] [lent @y 'a': list[u16], lent @y 'b': list[u16], lent @x 'c': list[u16]] -> lent @x list[u16] {
    return lend 'c';
}

MAIN {
    let mut 'xs': list[i32] = [5, -6, 7, 8];
    put_i[to.i64[list.remove[lend.mut 'xs', pos: 2]]];
    let 'kept' := list.remove[lend.mut 'xs', pos: 1];
    put_i[to.i64['kept']];
    list.remove[lend.mut 'xs', pos: list.length[lend 'xs']];
    put_u[list.length[lend 'xs']];
    put_i[to.i64['xs'[pos: 1]]];
}
