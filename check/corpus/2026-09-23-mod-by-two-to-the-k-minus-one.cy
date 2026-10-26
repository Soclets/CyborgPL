# Finding 2026-09-23 (generator seed 106 with inputs, shrunk): 'in1' mod 255 and
# 'in1' mod 65535 give 0 in CCA's binary (c845110) when 'in1' is 27796; other divisors are right.
# env CHK_IN1 27796
# check claude generator, seed 106
%extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
%extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
%extern.C fn strlen ['s': ptr[u8]] -> u64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append(lend.mut 'buf', value: 45); }
    let mut 'v': u64 = 'm';
    let mut 'ds': std::list[u8] = [];
    std::list.append(lend.mut 'ds', value: to.u8(48 plus ('v' mod.floor 10)));
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        std::list.append(lend.mut 'ds', value: to.u8(48 plus ('v' mod.floor 10)));
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': u64 = std::list.length(lend 'ds');
    while 'k' at_least 1 {
        std::list.append(lend.mut 'buf', value: 'ds'[pos: 'k']);
        change 'k' = 'k' minus 1;
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
    let 'k' := "CHK_IN1";
    no_check let 'in1': u64 = strlen(getenv(lend 'k'));
    put_u('in1' mod.floor 100);
    put_u('in1' mod.floor 255);
    put_u('in1' mod.floor 256);
    put_u('in1' mod.floor 1000);
    put_u('in1' mod.floor 30000);
    put_u('in1' mod.floor 27797);
    put_u('in1' mod.floor 65535);
    put_u('in1' mod.floor 65536);
    put_u('in1' mod.floor 100000);
}
