# Finding 2026-09-24 (exhaustive run, seed 3061, shrunk): a bare-number sum past 64 bits,
# (18446744073709551615 + 1) mod 1, stops CCA's binary inside a loop. Owner, dc83c05:
# bare numbers alone are exact, so this is 0.
# env CHK_IN1 0
# env CHK_IN2 0
# check claude generator, seed 3061
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
    let 'v20': u64 = 0;
    let 'w23': u16 = 2;
    loop {
        if 'w23' equal_to 0 {
            break;
        }
        change 'w23' = 'w23' minus 1;
        change 'v20' = (18446744073709551615 plus 1) mod.floor 1;
    }
}
