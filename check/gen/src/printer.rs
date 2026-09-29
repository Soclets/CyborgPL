//! The program's own printer (owner, 2026-09-23: every generated program
//! carries one, written in the base language, until std has a print). It is
//! ordinary CyborgPL and is compared like the rest, never special-cased.
//! Varied by seed so it does not always test the same path.

use crate::rng::Rng;

pub fn printer(rng: &mut Rng) -> String {
    let digits = match rng.below(3) {
        0 => DIGITS_REVERSED_FOR,
        1 => DIGITS_REVERSED_WHILE,
        _ => DIGITS_POWER_OF_TEN,
    };
    let put_i = if rng.chance(50) { PUT_I_IF } else { PUT_I_ELSE_FIRST };
    format!(
        "fn emit ['m': uint64, 'neg': bool] {{\n    mut 'buf': list[uint8] = [];\n    if 'neg' {{ list.append[lend.mut 'buf', 45]; }}\n{digits}    list.append[lend.mut 'buf', 10];\n    nocheck write[1, lend 'buf', list.length[lend 'buf']];\n}}\n\nfn put_u ['n': uint64] {{\n    emit['n', false];\n}}\n\n{put_i}"
    )
}

const DIGITS_REVERSED_FOR: &str = "    mut 'v': uint64 = 'm';
    mut 'ds': list[uint8] = [];
    loop {
        list.append[lend.mut 'ds', to.uint8[48 + ('v' mod 10)]];
        change 'v' = 'v' div 10;
        if 'v' == 0 { break; }
    }
    for 'k': uint64 in range[list.length[lend 'ds'], 1, -1] {
        list.append[lend.mut 'buf', 'ds'['k']];
    }
";

const DIGITS_REVERSED_WHILE: &str = "    mut 'v': uint64 = 'm';
    mut 'ds': list[uint8] = [];
    list.append[lend.mut 'ds', to.uint8[48 + ('v' mod 10)]];
    change 'v' = 'v' div 10;
    while 'v' > 0 {
        list.append[lend.mut 'ds', to.uint8[48 + ('v' mod 10)]];
        change 'v' = 'v' div 10;
    }
    mut 'k': uint64 = list.length[lend 'ds'];
    while 'k' >/== 1 {
        list.append[lend.mut 'buf', 'ds'['k']];
        change 'k' = 'k' - 1;
    }
";

const DIGITS_POWER_OF_TEN: &str = "    mut 'v': uint64 = 'm';
    mut 'p': uint64 = 1;
    while 'v' div 'p' >/== 10 {
        change 'p' = 'p' x 10;
    }
    loop {
        list.append[lend.mut 'buf', to.uint8[48 + ('v' div 'p')]];
        change 'v' = 'v' mod 'p';
        if 'p' == 1 { break; }
        change 'p' = 'p' div 10;
    }
";

const PUT_I_IF: &str = "fn put_i ['n': int64] {
    if 'n' < 0 {
        emit[to.uint64[0 - ('n' + 1)] + 1, true];
    } else {
        emit[to.uint64['n'], false];
    }
}
";

const PUT_I_ELSE_FIRST: &str = "fn put_i ['n': int64] {
    if 'n' >/== 0 {
        emit[to.uint64['n'], false];
    } else {
        'up': int64 = -('n' + 1);
        emit[to.uint64['up'] + 1, true];
    }
}
";
