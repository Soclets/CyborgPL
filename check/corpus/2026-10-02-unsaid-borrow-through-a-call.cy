# Finding 2026-10-02 (check run, seed 10731, cut down by hand): g hands back the borrow f returns, but
# its signature says an owned list (-> list[int8]). CCA builds it: the binary prints 0 for 'r''s length
# and aborts (exit 134, as if both 'r' and 's' freed one list). The oracle refuses it: `g` returns
# a borrow, so its signature must say so. CCA does refuse a direct `return lend 'a'` under -> list[T].
# check claude generator, seed 1
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;

fn emit ['m': uint64, 'neg': bool] {
    let mut 'buf': list[uint8] = [];
    if 'neg' { list.append[lend.mut 'buf', value 45]; }
    let mut 'v': uint64 = 'm';
    let mut 'ds': list[uint8] = [];
    list.append[lend.mut 'ds', value to.uint8[48 plus ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        list.append[lend.mut 'ds', value to.uint8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': uint64 = list.length[lend 'ds'];
    while 'k' at_least 1 {
        list.append[lend.mut 'buf', value 'ds'[pos 'k']];
        change 'k' = 'k' minus 1;
    }
    list.append[lend.mut 'buf', value 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', list.length[lend 'buf']];
}

fn put_u ['n': uint64] {
    emit['n', false];
}

fn put_i ['n': int64] {
    if 'n' at_least 0 {
        emit[to.uint64['n'], false];
    } else {
        let 'up': int64 = -('n' plus 1);
        emit[to.uint64['up'] plus 1, true];
    }
}

fn f [lent 'a': list[int8]] -> lent list[int8] {
    return lend 'a';
}
fn g [lent 'a': list[int8]] -> list[int8] {
    return f[lend 'a'];
}
MAIN {
    let mut 's': list[int8] = [1, 2, 3];
    let 'r' := g[lend 's'];
    list.append[lend.mut 's', value 4];
    put_u[list.length[lend 'r']];
    put_u[list.length[lend 's']];
}
