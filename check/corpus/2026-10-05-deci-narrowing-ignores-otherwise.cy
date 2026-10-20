# Finding 2026-10-05 (check run, seed 18003, cut down by hand): a d128 holding 1e400 converted
# with `to.d32['a'] otherwise -180.0`. A d32 cannot hold it, so the conversion fails and the
# `otherwise` gives -180.0: the oracle prints -180000. The CCA binary stops instead: "check: the
# value is past what a `d32` holds".
# check claude generator, seed 18003
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

MAIN {
    let 'a' := to.d128["1e400"] otherwise -21.363;
    let 'b' := to.d32['a'] otherwise -180.0;
    let 'z' := to.i64['b' times 1000] otherwise -1;
    put_i['z'];
}
