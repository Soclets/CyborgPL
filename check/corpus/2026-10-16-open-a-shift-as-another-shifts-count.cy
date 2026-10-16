# Finding 2026-10-16 (a width probe, by hand): a shift of a bare number used as another shift's count,
# to.int64[1] shl (5 shr to.uint8[1]). The oracle runs it (prints 4 then 7: the count is 2); kayo
# refuses it: "`shl` works on integers, and this is a `(not yet known)`", a message that names no
# rule. 4866c834 says a shift's width "never" comes from its count; whether a count's place gives a
# shift inside it a width is not ruled. Open: the owner's.
# check claude generator (printer only)
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
MAIN {
    let 'r': int64 = to.int64[1] shl (5 shr to.uint8[1]);
    put_i['r'];
    put_u[7];
}
