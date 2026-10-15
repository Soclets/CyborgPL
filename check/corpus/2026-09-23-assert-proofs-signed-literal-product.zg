# Finding 2026-09-23 (generator seed 22 with inputs, shrunk): --assert-proofs
# asserts 2 x 7726793855274730819 (a uint64 argument) as a signed multiply.
# env CHK_IN1 0
# env CHK_IN2 0
# check claude generator, seed 22
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;

fn emit ['m': uint64, 'neg': bool] {
    let mut 'buf': list[uint8] = [];
    if 'neg' { list.append[lend.mut 'buf', value 45]; }
    let mut 'v': uint64 = 'm';
    let mut 'ds': list[uint8] = [];
    loop {
        list.append[lend.mut 'ds', value to.uint8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
        if 'v' equal_to 0 { break; }
    }
    for 'k': uint64 from list.length[lend 'ds'] to 1 by -1 {
        list.append[lend.mut 'buf', value 'ds'[pos 'k']];
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

fn f3 ['p1': uint32, 'p2': uint16, 'p3': int32] -> uint16 {
    let 'w6': int32 = 1;
    while 'w6' > 0 {
        change 'w6' = 0;
        put_u[2 times 7726793855274730819];
    }
    return 0;
}

MAIN {
    let 'in1_key' := "CHK_IN1";
    no_check let 'in1': uint64 = strlen[getenv[lend 'in1_key']];
    let 'in2_key' := "CHK_IN2";
    no_check let 'in2': uint64 = strlen[getenv[lend 'in2_key']];
    if 0 < f3[0, 1, 0] {
    }
}
