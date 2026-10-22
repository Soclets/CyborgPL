# Finding 2026-10-02 (check run, seed 10731, cut down by hand): g hands back the borrow f returns, but
# its signature says an owned list (-> list[i8]). CCA builds it: the binary prints 0 for 'r''s length
# and aborts (exit 134, as if both 'r' and 's' freed one list). The oracle refuses it: `g` returns
# a borrow, so its signature must say so. CCA does refuse a direct `return lend 'a'` under -> list[T].
# check claude generator, seed 1
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'ds': std::list[u8] = [];
    std::list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        std::list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': u64 = std::list.length[lend 'ds'];
    while 'k' at_least 1 {
        std::list.append[lend.mut 'buf', value: 'ds'[pos: 'k']];
        change 'k' = 'k' minus 1;
    }
    std::list.append[lend.mut 'buf', value: 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', std::list.length[lend 'buf']];
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

fn f [lent 'a': std::list[i8]] -> lent std::list[i8] {
    return lend 'a';
}
fn g [lent 'a': std::list[i8]] -> std::list[i8] {
    return f[lend 'a'];
}
MAIN {
    let mut 's': std::list[i8] = [1, 2, 3];
    let 'r' := g[lend 's'];
    std::list.append[lend.mut 's', value: 4];
    put_u[std::list.length[lend 'r']];
    put_u[std::list.length[lend 's']];
}
