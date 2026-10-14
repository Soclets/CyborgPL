# Finding 2026-09-23 (generator seed 133, shrunk): 0 - 'in2' (uint64, 'in2' = 2) in a
# loop after an empty for: the oracle stops, CCA's binary runs on.
# env CHK_IN1 67757
# env CHK_IN2 2
# check claude generator, seed 133
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
    let 'in1_key' := "CHK_IN1";
    no_check let 'in1': uint64 = strlen[getenv[lend 'in1_key']];
    let 'in2_key' := "CHK_IN2";
    no_check let 'in2': uint64 = strlen[getenv[lend 'in2_key']];
    for 'i21': int16 from 37 to 40 by 1 {
    }
    let 'w24': uint16 = 4;
    loop {
        if 'w24' at_most 0 {
            break;
        }
        change 'w24' = 'w24' minus 1;
        let 'v25': bool = 0 minus 'in2' at_least 0;
    }
}
