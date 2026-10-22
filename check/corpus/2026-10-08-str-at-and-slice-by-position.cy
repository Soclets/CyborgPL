# Coverage 2026-10-08 (owner: "yea, ask it to cover them"): str.at in each unit and std::str.slice.grapheme,
# with pos: (f2f3879b), in range and out of it (the otherwise). CCA has no stdkit yet: "not yet" there.
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

fn b1 [@y, @x] [lent @y 'a': std::list[u16], lent @y 'b': std::list[u16], lent @x 'c': std::list[u16]] -> lent @x std::list[u16] {
    return lend 'c';
}

fn b2 [@x, @y] [lent 'a': std::list[u16], lent @x 'b': std::list[u16], lent @y 'c': std::list[u16]] -> lent @x std::list[u16] {
    return b1[lend 'b', lend 'a', lend 'b'];
}

fn b3 [@y, @x] [lent @y 'a': std::list[u16], lent @x 'b': std::list[u16], lent @x 'c': std::list[u16]] -> lent @x std::list[u16] {
    if std::list.length[lend 'c'] at_least std::list.length[lend 'a'] {
        return b2[lend 'a', lend 'c', lend 'c'];
    }
    return b1[lend 'c', lend 'c', lend 'b'];
}

fn b4 [@y, @x] [lent @y 'a': std::list[u16], lent @y 'b': std::list[u16], lent @x 'c': std::list[u16]] -> lent @x std::list[u16] {
    return lend 'c';
}

MAIN {
    let 's': str.utf8 = "hÃ©llo";
    let 'b' := std::str.at.byte[lend 's', pos: 2] otherwise 7;
    put_u[to.u64['b']];
    let 'c' := std::str.at.point[lend 's', pos: 2] otherwise `?`;
    put_u[to.u64[char.code['c']]];
    let 'g' := std::str.at.grapheme[lend 's', pos: 9] otherwise `?`;
    std::print.useful['g', newln];
    let 'sl' := std::str.slice.grapheme[lend 's', from: pos 2, to: pos 4] otherwise str.empty;
    put_u[std::str.bytes[lend 'sl']];
    let 'none' := std::str.slice.grapheme[lend 's', from: pos 4, to: pos 9] otherwise "-";
    put_u[std::str.bytes[lend 'none']];
}
