# Finding 2026-10-06 (check run with nothing avoided, seeds 25263, 25528, 25741, 25784, 26052, 26090,
# cut down by hand): 'v' xor (15803055715966303544 minus 8) with a u64 'v', into a u64. The bare
# group is 15803055715966303536, which a u64 holds; in between, the one limit is a value no 64-bit
# type holds (design/operators.md). The oracle prints 1. CCA refuses: "`minus` can leave what the
# type holds ... a `integer` holds -9223372036854775808 to 9223372036854775807". Beside `plus`, or
# stored alone, it builds; beside or.bit and xor it does not. The same for `plus` and `times` groups.
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
    let 'v': u64 = 5;
    let 'r': u64 = 'v' xor (15803055715966303544 minus 8);
    put_u['r' mod.floor 7];
}
