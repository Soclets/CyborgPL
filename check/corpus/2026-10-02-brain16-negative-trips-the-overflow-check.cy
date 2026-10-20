# Finding 2026-10-02 (check run, seeds 12026 and 14 more, cut down by hand): a negative brain16 value
# trips CCA's run-time overflow check: -0.3 x 10.0 is -3 (the oracle prints -3, then -7 for -1.75 x 4.0),
# but the CCA binary stops: "`x` can go past the largest brain16 (it would be infinity)". The same with
# `+`, `-` and `div` on negative values (-1.75 + -1.75); positive values agree.
# check claude generator, seed 11045
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': list[u8] = [];
    if 'neg' { list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'p': u64 = 1;
    while 'v' div.floor 'p' at_least 10 {
        change 'p' = 'p' times 10;
    }
    loop {
        list.append[lend.mut 'buf', value: to.u8[48 plus ('v' div.floor 'p')]];
        change 'v' = 'v' mod.floor 'p';
        if 'p' equal_to 1 { break; }
        change 'p' = 'p' div.floor 10;
    }
    list.append[lend.mut 'buf', value: 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', list.length[lend 'buf']];
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
MAIN {
    let 'a': brain16 = -0.3;
    let 'z' := to.i64['a' times 10.0] otherwise -1;
    put_i['z'];
    let 'b': brain16 = -1.75;
    let 'y' := to.i64['b' times 4.0] otherwise -1;
    put_i['y'];
}
