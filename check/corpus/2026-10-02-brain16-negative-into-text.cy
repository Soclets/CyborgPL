# Finding 2026-10-02 (check run, seed 12121, cut down by hand): a negative brain16 value into text, in
# either form. The oracle's lengths: 0.3 → 3 ("0.3") and 10 stored, -0.3 → 4 and 11, 2.5 → 3 and 3,
# -2.5 → 4 and 4, -1.75 → 5 and 5. CCA gives 1 for every negative value, in both forms; positive ones agree.
# check claude generator, seed 11045
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

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
    NO_FUCKING_CHECKS write(1, lend 'buf', std::list.length(lend 'buf'));
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
    let 'a1': brain16 = 0.3;
    let 'u1' := to.useful.str.utf8('a1');
    put_u(std::str.bytes(lend 'u1'));
    let 's1' := to.stored.str.utf8('a1');
    put_u(std::str.bytes(lend 's1'));
    let 'a2': brain16 = -0.3;
    let 'u2' := to.useful.str.utf8('a2');
    put_u(std::str.bytes(lend 'u2'));
    let 's2' := to.stored.str.utf8('a2');
    put_u(std::str.bytes(lend 's2'));
    let 'a3': brain16 = 2.5;
    let 'u3' := to.useful.str.utf8('a3');
    put_u(std::str.bytes(lend 'u3'));
    let 's3' := to.stored.str.utf8('a3');
    put_u(std::str.bytes(lend 's3'));
    let 'a4': brain16 = -2.5;
    let 'u4' := to.useful.str.utf8('a4');
    put_u(std::str.bytes(lend 'u4'));
    let 's4' := to.stored.str.utf8('a4');
    put_u(std::str.bytes(lend 's4'));
    let 'a5': brain16 = -1.75;
    let 'u5' := to.useful.str.utf8('a5');
    put_u(std::str.bytes(lend 'u5'));
    let 's5' := to.stored.str.utf8('a5');
    put_u(std::str.bytes(lend 's5'));
}
