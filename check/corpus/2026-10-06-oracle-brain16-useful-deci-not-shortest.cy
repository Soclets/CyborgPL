# Finding 2026-10-06 (check run, seed 25294, cut down by hand): a brain16 of 2^63, from an i64 just
# below it. to.useful.d64 is the shortest decimal that reads back as the same value: 9.22e18 at
# brain16's precision, which the oracle's own print.useful shows (9220000000000000000) and which fits
# an i64, so CCA prints 9220000000000000000. The oracle's to.useful.d64 gives 9223372036854776000
# (its stored form), past i64, so its to.i64[...] otherwise -1 prints -1.
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

fn b1 [@x] [lent @x 'a': std::list[i16], lent @x 'b': std::list[i16]] -> lent @x std::list[i16] {
    if std::list.length(lend 'b') > 2 {
        return lend 'b';
    }
    return lend 'a';
}

fn b2 [@x] [lent @x 'a': std::list[i16]] -> lent @x std::list[i16] {
    if std::list.length(lend 'a') > 2 {
        return b1(lend 'a', lend 'a');
    }
    return lend 'a';
}

fn b3 [@x] [lent @x 'a': std::list[i16], lent @x 'b': std::list[i16]] -> lent @x std::list[i16] {
    return lend 'a';
}

MAIN {
    let 'v': i64 = 9223372036854775803;
    let 'w' := to.brain16('v');
    let 'd' := to.useful.d64('w');
    let 'z' := to.i64('d') otherwise -1;
    put_i('z');
}
