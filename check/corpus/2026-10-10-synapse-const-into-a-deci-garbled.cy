# Finding 2026-10-10 (check run, seed 48186, cut down by hand): a body-level const (owner, 2026-10-10)
# converted to a d128. const 'c': i64 = 1234, then to.d128['c']: the oracle has 1234, and
# to.i64['d' times 1] gives 1234. Synapse's deci comes out wrong (printed stored, 1234 shows as
# 0.000…0001234 with a huge negative exponent; -9223372036854775807 shows as -0), so it gives -1. The
# same value from a `let` converts right.
# check claude generator, seed 1
%extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

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
    NO_FUCKING_CHECKS write(1, lend 'buf', std::list.length(lend 'buf'));
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

struct Counter {
    now: i32,
    stop: i32
}

fn Counter.next [lent.mut 'c': Counter] -> optional[i32] {
    if 'c'.now at_least 'c'.stop { return optional.none; }
    change 'c'.now = 'c'.now plus 1;
    return optional.value('c'.now);
}

fn half ['n': i32] -> optional[i32] {
    if ('n' mod.floor 2) equal_to 0 { return optional.value('n' div.floor 2); }
    return optional.none;
}

MAIN {
    const 'c': i64 = 1234;
    let 'd' := to.%d128('c');
    let 'z' := to.i64('d' times 1) otherwise -1;
    put_i('z');
}
