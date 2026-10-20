# Finding 2026-09-23 (generator seed 371, shrunk): --assert-proofs trips 'the divisor
# can be zero' on 0 / ((2 / 10047273927188643194) + 1), whose divisor is always 1.
# env CHK_IN1 0
# env CHK_IN2 0
# check claude generator, seed 371
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn strlen ['s': ptr[u8]] -> u64;

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

fn f3 ['p1': bool] -> i16 {
    put_u[to.u64[0 div.floor ((2 div.floor 10047273927188643194) plus 1)]];
    return 0;
}

MAIN {
    let 'in1_key' := "CHK_IN1";
    no_check let 'in1': u64 = strlen[getenv[lend 'in1_key']];
    let 'in2_key' := "CHK_IN2";
    no_check let 'in2': u64 = strlen[getenv[lend 'in2_key']];
    put_i[to.i64[f3[true]]];
}
