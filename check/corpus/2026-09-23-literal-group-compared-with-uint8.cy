# Finding 2026-09-23 (generator seed 77, shrunk): a literal group compared with a
# u8 takes the wrong branch in CCA (0434f04): 206 - 8 > 201 is false.
# env CHK_IN1 12745
# check claude generator, seed 77
%extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
%extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
%extern.C fn strlen ['s': ptr[u8]] -> u64;

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
    if 'n' at_least 0 {
        emit(to.u64('n'), false);
    } else {
        let 'up': i64 = -('n' plus 1);
        emit(to.u64('up') plus 1, true);
    }
}

MAIN {
    let 'k' := "CHK_IN1";
    no_check let 'in1': u64 = strlen(getenv(lend 'k'));
    let 'u': u8 = to.u8('in1' mod.floor 256);
    if 206 minus 8 > 'u' { put_i(0); }
}
