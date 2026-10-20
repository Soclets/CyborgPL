# Finding 2026-09-23 (generator seed 133, shrunk): 0 - 'in2' (u64, 'in2' = 2) in a
# loop after an empty for: the oracle stops, CCA's binary runs on.
# env CHK_IN1 67757
# env CHK_IN2 2
# check claude generator, seed 133
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn strlen ['s': ptr[u8]] -> u64;

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
    let 'in1_key' := "CHK_IN1";
    no_check let 'in1': u64 = strlen[getenv[lend 'in1_key']];
    let 'in2_key' := "CHK_IN2";
    no_check let 'in2': u64 = strlen[getenv[lend 'in2_key']];
    for 'i21': i16 [from: 37, to: 40, by: 1] {
    }
    let 'w24': u16 = 4;
    loop {
        if 'w24' at_most 0 {
            break;
        }
        change 'w24' = 'w24' minus 1;
        let 'v25': bool = 0 minus 'in2' at_least 0;
    }
}
