# Finding 2026-09-23 (generator seed 316, shrunk structurally): a check CCA refuses,
# 'divisor can be zero' on not.bit 'v2' with 'v2' in [-102, -96] across two loops.
# Fixed 2026-09-23 in CCA 0279bf8 (layer 2 widened at every busy block; no range for not.bit).
# env CHK_IN1 0
# env CHK_IN2 0
# check claude generator, seed 316
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn strlen ['s': ptr[u8]] -> u64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append(lend.mut 'buf', value: 45); }
    let mut 'v': u64 = 'm';
    let mut 'ds': std::list[u8] = [];
    loop {
        std::list.append(lend.mut 'ds', value: to.u8(48 plus ('v' mod.floor 10)));
        change 'v' = 'v' div.floor 10;
        if 'v' equal_to 0 { break; }
    }
    for 'k': u64 [from: std::list.length(lend 'ds'), to: 1, by: -1] {
        std::list.append(lend.mut 'buf', value: 'ds'[pos: 'k']);
    }
    std::list.append(lend.mut 'buf', value: 10);
    no_check write(1, lend 'buf', std::list.length(lend 'buf'));
}

fn put_u ['n': u64] {
    emit('n', false);
}

fn put_i ['n': i64] {
    if 'n' at_least 0 {
        emit(to.u64('n'), false);
    } else {
        let 'up': i64 = -('n' plus 1);
        emit(to.u64('up') plus 1, true);
    }
}

MAIN {
    let 'in1_key' := "CHK_IN1";
    no_check let 'in1': u64 = strlen(getenv(lend 'in1_key'));
    let 'in2_key' := "CHK_IN2";
    no_check let 'in2': u64 = strlen(getenv(lend 'in2_key'));
    let immut 'v1': i64 = to.i64('in1' mod.floor 9223372036854775808);
    let 'v2': i8 = (to.i8(('v1' mod.floor 256) minus 128) mod.floor 7) minus 102;
    let mut 'w3': i16 = 7;
    while 'w3' > 0 {
        change 'w3' = 'w3' minus 1;
        for 'i4': u16 [from: 29, to: 3, by: -3] {
            check change 'v2' = ((((126 or.bit 'v2') div.floor (not.bit 'v2')) shl 6) mod.floor 7) minus 102;
        }
    }
}
