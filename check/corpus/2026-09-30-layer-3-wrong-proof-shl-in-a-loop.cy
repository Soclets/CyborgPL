# Finding 2026-09-30 (check run, seed 9148, cut down by hand): a wrong layer-3 proof. 'w10' counts down
# 5..0 in a loop, and 2030534677 shl 'w10' overflows u32 for every count past 1. Precog claims the
# shift proven (shift_width, layer 3, solver) with its own operands {2030534677} and {0..5}; the binary
# prints wrapped values (552600224 = 64977109664 mod 2^32 first). The oracle stops at the first shift.
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

fn f1 ['p1': i64] -> i32 {
    let mut 'w10': i8 = 6;
    loop {
        if 'w10' equal_to 0 {
            break;
        }
        change 'w10' = 'w10' minus 1;
        put_u(to.u32('p1' mod.floor 4294967296) shl to.u32('w10'));
    }
    return 0;
}
MAIN {
    let 'x': i32 = f1(2030534677);
}
