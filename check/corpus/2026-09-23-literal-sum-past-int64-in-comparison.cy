# Finding 2026-09-23 (generator seed 439, shrunk): CCA traps on a literal sum of
# 1.17e19 in a comparison. Owner, dc83c05: plain math is exact up to 64 bits, so
# this compares true values and runs.
# env CHK_IN1 0
# env CHK_IN2 0
# check claude generator, seed 439
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn strlen ['s': ptr[u8]] -> u64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append(lend.mut 'buf', value: 45); }
    let mut 'v': u64 = 'm';
    let mut 'p': u64 = 1;
    while 'v' div.floor 'p' at_least 10 {
        change 'p' = 'p' times 10;
    }
    loop {
        std::list.append(lend.mut 'buf', value: to.u8(48 plus ('v' div.floor 'p')));
        change 'v' = 'v' mod.floor 'p';
        if 'p' equal_to 1 { break; }
        change 'p' = 'p' div.floor 10;
    }
    std::list.append(lend.mut 'buf', value: 10);
    no_check write(1, lend 'buf', std::list.length(lend 'buf'));
}

fn put_u ['n': u64] {
    emit('n', false);
}

fn put_i ['n': i64] {
    if 'n' < 0 {
        emit(to.u64(0 minus ('n' plus 1)) plus 1, true);
    } else {
        emit(to.u64('n'), false);
    }
}

MAIN {
    let 'in1_key' := "CHK_IN1";
    no_check let 'in1': u64 = strlen(getenv(lend 'in1_key'));
    let 'in2_key' := "CHK_IN2";
    no_check let 'in2': u64 = strlen(getenv(lend 'in2_key'));
    if 'in2' equal_to 1 {
    } else if 3876992840477302775 plus 7782534087963747351 > 0 {
    }
}
