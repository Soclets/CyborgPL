# Finding 2026-10-16 (a width probe, by: hand, after 59c45ac6): two more places where a shift of a bare
# number has no width (a conversion is not a context, 2026-10-07 corpus; print gives none) that the
# oracle refuses only where they run. In a branch never taken, kayo refuses `to.i64[5 shr
# to.u8[1]]` when building; the oracle prints 7. Taken, the oracle refuses it ("`shr` needs a
# width"), and the same for std::print.useful[5 shr to.u8[1], newln].
# check claude generator (printer only)
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
MAIN {
    let 'p': bool = to.i64(1) equal_to to.i64(2);
    if 'p' {
        let 'r' := to.i64(5 shr to.u8(1));
        std::print.useful(5 shr to.u8(1), newln);
    }
    put_u(7);
}
