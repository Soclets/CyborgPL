# Finding 2026-09-29 (check run, seed 3593, shrunk): f2[0, 1, 0] goes straight to the base case with
# 'p1' = 1, so the divisor -to.i64['p1'] is -1 (and 448..680 from the self-call). CCA refuses it:
# "the divisor is always zero". The oracle runs it.
# check claude generator, seed 3593
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
    no_check write[1, lend 'buf', list.length[lend 'buf']];
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

enum E1 {
    c1,
    c2: u64
}

enum E2 {
    c1,
    c2: i16,
    c3: u16
}

fn f2 ['pr': i32[min: 0, max: 6], 'p1': u64, 'p2': u64[max: 66]] -> i64 {
    if 'pr' equal_to 0 {
        return (((to.i64[to.i64['p1'] or.bit 15] or.bit 1) times ((2855519550069594396 minus 6) mod.floor (-to.i64['p1']))) mod.floor 1295) minus 800;
    }
    let 'r3': i64 = f2['pr' minus 1, ((('p1' times 'p1') minus 'p2') mod.floor 233) plus 448, 'p1' mod.floor 67];
    return 'r3';
}

MAIN {
    let 'v12': i64 = to.i64[f2[0, 1, 0]] mod.floor 1;
}
