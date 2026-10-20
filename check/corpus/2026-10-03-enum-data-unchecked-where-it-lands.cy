# Finding 2026-10-03 (check run, seed 15174, cut down by hand): enum case data is not checked where
# it lands. E1.c4 carries a u64, given 0 - 5. The oracle stops: "-5 does not fit u64". The CCA
# binary runs and the match prints 18446744073709551611 (2^64 - 5, wrapped). The same with an i64
# name holding -5 as the data. In the seed it was bare numbers worth about -2 x 10^37.
# check claude generator, seed 15174
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': list[u8] = [];
    if 'neg' { list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'ds': list[u8] = [];
    loop {
        list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
        if 'v' equal_to 0 { break; }
    }
    for 'k': u64 [from: list.length[lend 'ds'], to: 1, by: -1] {
        list.append[lend.mut 'buf', value: 'ds'[pos: 'k']];
    }
    list.append[lend.mut 'buf', value: 10];
    no_check write[1, lend 'buf', list.length[lend 'buf']];
}

fn put_u ['n': u64] {
    emit['n', false];
}

fn put_i ['n': i64] {
    if 'n' < 0 {
        emit[to.u64[0 minus ('n' plus 1)] plus 1, true];
    } else {
        emit[to.u64['n'], false];
    }
}

enum E1 {
    c1: u32,
    c2,
    c3,
    c4: u64
}

layout.C enum E2 {
    c1 = -169,
    c2 = -2003713270
}

MAIN {
    let 'e' := E1.c4[0 minus 5];
    match 'e' {
        c1 'a' { put_u[to.u64['a']]; }
        c2 { put_u[2]; }
        c3 { put_u[3]; }
        c4 'b' { put_u['b']; }
    }
}
