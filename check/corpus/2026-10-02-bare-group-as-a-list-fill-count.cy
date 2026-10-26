# Finding 2026-10-02 (check run --borrow-wrong 1000, seed 11045, cut down by hand): a count of bare
# numbers alone, ((-9223372036854775808) x 4611686018427387904) mod 9, is -2^125 mod 9 = 4, worked out
# exactly at compile time (design/operators.md). The oracle makes 4 elements. CCA refuses it here
# ("`x` can leave what the type holds"; in the full program the binary stopped there instead). The
# same group stored by a plain `let 'n': u64 = …` agrees, so it is list.fill's count.
# check claude generator, seed 11045
%extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

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
    let 'xs': std::list[i16] = std::list.fill(value: -336, count: ((-9223372036854775808) times 4611686018427387904) mod.floor 9);
    put_u(std::list.length(lend 'xs'));
}
