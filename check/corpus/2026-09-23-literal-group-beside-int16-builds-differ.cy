# Finding 2026-09-23 (generator seed 25, shrunk). Owner, dc83c05: plain math is
# exact until it lands, so the literal group is -51967 and put_i prints 8..0.
# env CHK_IN1 0
# env CHK_IN2 0
# check claude generator, seed 25
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn strlen ['s': ptr[u8]] -> u64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'p': u64 = 1;
    while 'v' div.floor 'p' at_least 10 {
        change 'p' = 'p' times 10;
    }
    loop {
        std::list.append[lend.mut 'buf', value: to.u8[48 plus ('v' div.floor 'p')]];
        change 'v' = 'v' mod.floor 'p';
        if 'p' equal_to 1 { break; }
        change 'p' = 'p' div.floor 10;
    }
    std::list.append[lend.mut 'buf', value: 10];
    no_check write[1, lend 'buf', std::list.length[lend 'buf']];
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

MAIN {
    let 'in1_key' := "CHK_IN1";
    no_check let 'in1': u64 = strlen[getenv[lend 'in1_key']];
    let 'in2_key' := "CHK_IN2";
    no_check let 'in2': u64 = strlen[getenv[lend 'in2_key']];
    let 'w7': i32 = 9;
    loop {
        if 'w7' equal_to 0 {
            break;
        }
        change 'w7' = 'w7' minus 1;
        put_i[(((-28014) plus (-23953)) mod.floor 1) plus to.i16['w7']];
    }
}
