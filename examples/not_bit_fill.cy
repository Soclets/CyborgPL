# Finding 2026-10-05 (check run --borrow-wrong 1000 --avoid none, seeds 24107 and 24126, shrunk and
# cut down by hand): `simd.fill[value not.bit (5 plus 'v17')]` with a u16 'v17', for a simd[u16, count 8].
# The width comes from any operand whose type is known (owner: a, 2026-09-23), here 'v17', and the
# lanes are u16 besides. The oracle stops: "`not.bit` flips a type's bits, so it needs to know how
# many". With ('v17' plus 5) it runs, as it does for not.bit (5 plus 'v17') in a comparison or a
# print: inside simd.fill it looks at the left operand only. kayo runs it.
# check claude generator, seed 24126
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': list[u8] = [];
    if 'neg' { list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'ds': list[u8] = [];
    list.append[lend.mut 'ds', value: to.u8[48 + ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        list.append[lend.mut 'ds', value: to.u8[48 + ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': u64 = list.length[lend 'ds'];
    while 'k' at_least 1 {
        list.append[lend.mut 'buf', value: 'ds'[pos: 'k']];
        change 'k' = 'k' - 1;
    }
    list.append[lend.mut 'buf', value: 10];
    no_check write[1, lend 'buf', list.length[lend 'buf']];
}

fn put_u ['n': u64] {
    emit['n', false];
}

fn put_i ['n': i64] {
    if 'n' < 0 {
        emit[to.u64[0 - ('n' + 1)] + 1, true];
    } else {
        emit[to.u64['n'], false];
    }
}
MAIN {
    let 'v17': u16 = 0;
    let 's21': simd[u16, count: 8] = simd.fill[value: not.bit (5 + 'v17')];
    put_u[to.u64[simd.max['s21']]];
}
