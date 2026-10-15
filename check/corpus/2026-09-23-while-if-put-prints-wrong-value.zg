# Finding 2026-09-23 (generator seed 2 with inputs, shrunk): put_i[0] inside an if
# inside a one-trip while prints 1 in CCA's binary; the oracle prints 0.
# env CHK_IN2 7745
# check claude generator, seed 2
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;

fn emit ['m': uint64, 'neg': bool] {
    let mut 'buf': list[uint8] = [];
    if 'neg' { list.append[lend.mut 'buf', value 45]; }
    let mut 'v': uint64 = 'm';
    let mut 'p': uint64 = 1;
    while 'v' div.floor 'p' at_least 10 {
        change 'p' = 'p' times 10;
    }
    loop {
        list.append[lend.mut 'buf', value to.uint8[48 plus ('v' div.floor 'p')]];
        change 'v' = 'v' mod.floor 'p';
        if 'p' equal_to 1 { break; }
        change 'p' = 'p' div.floor 10;
    }
    list.append[lend.mut 'buf', value 10];
    no_check write[1, lend 'buf', list.length[lend 'buf']];
}

fn put_u ['n': uint64] {
    emit['n', false];
}

fn put_i ['n': int64] {
    if 'n' < 0 {
        emit[to.uint64[0 minus ('n' plus 1)] plus 1, true];
    } else {
        emit[to.uint64['n'], false];
    }
}

MAIN {
    let 'k' := "CHK_IN2";
    no_check let 'n': uint64 = strlen[getenv[lend 'k']];
    let 'b': bool = 'n' > 0;
    let 'w': int64 = 1;
    while 'w' > 0 {
        change 'w' = 0;
        if 'b' { put_i[0]; }
    }
    put_i[1];
}
