# Finding 2026-10-07 (check run --borrow-wrong 1000, seed 28563, shrunk): if 0 >/== (-5988905564081113680)
# times (-4). Bare numbers alone are worked out exactly and a comparison compares true values: the
# product is 23955622256324454720, so the test is false and the oracle prints 1 (the else). The CCA
# binary prints 0: it takes the then branch, as if the product had wrapped.
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
    if 0 at_least (-5988905564081113680) times (-4) {
        put_u(0);
    } else {
        put_u(1);
    }
}
