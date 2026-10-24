# Finding 2026-09-29 (check run, seed 5120, shrunk then cut by hand): simd.fill[15059671820752541943 div 3]
# into simd[u64, count: 2] is 5019890606917513981 (bare numbers alone are exact). The normal build and the
# oracle agree; the --assert-proofs build stops (proof failed: the value can be outside what a u64
# holds), as if the literal past i64 were divided as a signed i64. A plain fill of the literal,
# or of a u64 name holding the quotient, is fine. The same family as assert-proofs-match-arm-bare-div.
# check claude generator, seed 1
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

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

MAIN {
    let 's': simd[u64, count: 2] = simd.fill(value: 15059671820752541943 div.floor 3);
    put_u(to.u64(simd.max('s')));
}
