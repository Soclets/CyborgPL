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
    let vouch = if rng.chance(50) { "nocheck" } else { "NO_FUCKING_CHECKS" };
    format!(
        "fn emit ['m': uint64, 'neg': bool] {{\n    let mut 'buf': list[uint8] = [];\n    if 'neg' {{ list.append[lend.mut 'buf', 45]; }}\n{digits}    list.append[lend.mut 'buf', 10];\n    {vouch} write[1, lend 'buf', list.length[lend 'buf']];\n}}\n\nfn put_u ['n': uint64] {{\n    emit['n', false];\n}}\n\n{put_i}"
    )
}

/// The printer of a `#strict` file: every site proven in its own function
/// (a digit from `mod 10`, the divisor under `if 'p' > 0`), the spelling of
/// strict files, and the one C call under `nocheck`.
pub fn strict_printer(rng: &mut Rng) -> String {
    let put_i = if rng.chance(50) { STRICT_PUT_I_IF } else { STRICT_PUT_I_ELSE_FIRST };
    let vouch = if rng.chance(50) { "nocheck" } else { "NO_FUCKING_CHECKS" };
    format!("fn emit ['m': uint64, 'neg': bool] {{\n    let mut 'buf': list[uint8] = [];\n    if 'neg' {{ list.append[lend.mut 'buf', 45]; }}\n{STRICT_DIGITS}    list.append[lend.mut 'buf', 10];\n    {vouch} write[1, lend 'buf', list.length[lend 'buf']];\n}}\n\nfn put_u ['n': uint64] {{\n    emit['n', false];\n}}\n\n{put_i}")
}

const STRICT_DIGITS: &str = "    let mut 'started': bool = false;
    let mut 'p': uint64 = 10000000000000000000;
    for temp 'k': int32 in range[from 1, to 20, by 1] {
        if 'p' > 0 {
            let immut 'd': uint64 = ('m' div.floor 'p') mod.floor 10;
            if 'd' !== 0 or 'started' or 'p' == 1 {
                change 'started' = true;
                list.append[lend.mut 'buf', to.uint8[48 + 'd']];
            }
            change 'p' = 'p' div.floor 10;
        }
    }
";

const STRICT_PUT_I_IF: &str = "fn put_i ['n': int64] {
    if 'n' < 0 {
        emit[to.uint64[0 - ('n' + 1)] + 1, true];
    } else {
        emit[to.uint64['n'], false];
    }
}
";

const STRICT_PUT_I_ELSE_FIRST: &str = "fn put_i ['n': int64] {
    if 'n' >/== 0 {
        emit[to.uint64['n'], false];
    } else {
        let immut 'up': int64 = -('n' + 1);
        emit[to.uint64['up'] + 1, true];
    }
}
";

const DIGITS_REVERSED_FOR: &str = "    let mut 'v': uint64 = 'm';
    let mut 'ds': list[uint8] = [];
    loop {
        list.append[lend.mut 'ds', to.uint8[48 + ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
        if 'v' == 0 { break; }
    }
    for 'k': uint64 in range[list.length[lend 'ds'], 1, -1] {
        list.append[lend.mut 'buf', 'ds'['k']];
    }
";

const DIGITS_REVERSED_WHILE: &str = "    let mut 'v': uint64 = 'm';
    let mut 'ds': list[uint8] = [];
    list.append[lend.mut 'ds', to.uint8[48 + ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        list.append[lend.mut 'ds', to.uint8[48 + ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': uint64 = list.length[lend 'ds'];
    while 'k' >/== 1 {
        list.append[lend.mut 'buf', 'ds'['k']];
        change 'k' = 'k' - 1;
    }
";

const DIGITS_POWER_OF_TEN: &str = "    let mut 'v': uint64 = 'm';
    let mut 'p': uint64 = 1;
    while 'v' div.floor 'p' >/== 10 {
        change 'p' = 'p' x 10;
    }
    loop {
        list.append[lend.mut 'buf', to.uint8[48 + ('v' div.floor 'p')]];
        change 'v' = 'v' mod.floor 'p';
        if 'p' == 1 { break; }
        change 'p' = 'p' div.floor 10;
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
        let 'up': int64 = -('n' + 1);
        emit[to.uint64['up'] + 1, true];
    }
}
";
