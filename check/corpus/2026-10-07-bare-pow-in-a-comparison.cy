# Finding 2026-10-07 (check run --risk 20, seed 27536, cut down by hand): 'v' !== 4207507906 pow 17,
# 'v' a u32. Bare numbers alone are worked out exactly, and a comparison compares true values
# (design/operators.md), so this is true; the oracle prints 1. CCA refuses: "`pow` goes past 64
# bits in between". Stored after a mod.floor ((4207507906 pow 17) mod.floor 1000) it builds (94dbca3).
# check claude generator, seed 1
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'ds': std::list[u8] = [];
    std::list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        std::list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': u64 = std::list.length[lend 'ds'];
    while 'k' at_least 1 {
        std::list.append[lend.mut 'buf', value: 'ds'[pos: 'k']];
        change 'k' = 'k' minus 1;
    }
    std::list.append[lend.mut 'buf', value: 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', std::list.length[lend 'buf']];
}

fn put_u ['n': u64] {
    emit['n', false];
}

fn put_i ['n': i64] {
    if 'n' at_least 0 {
        emit[to.u64['n'], false];
    } else {
        let 'up': i64 = -('n' plus 1);
        emit[to.u64['up'] plus 1, true];
    }
}

fn b1 [@x] [lent @x 'a': std::list[i16], lent @x 'b': std::list[i16]] -> lent @x std::list[i16] {
    if std::list.length[lend 'b'] > 2 {
        return lend 'b';
    }
    return lend 'a';
}

fn b2 [@x] [lent @x 'a': std::list[i16]] -> lent @x std::list[i16] {
    if std::list.length[lend 'a'] > 2 {
        return b1[lend 'a', lend 'a'];
    }
    return lend 'a';
}

fn b3 [@x] [lent @x 'a': std::list[i16], lent @x 'b': std::list[i16]] -> lent @x std::list[i16] {
    return lend 'a';
}

MAIN {
    let 'v': u32 = 5;
    if 'v' not_equal 4207507906 pow 17 { put_u[1]; }
}
