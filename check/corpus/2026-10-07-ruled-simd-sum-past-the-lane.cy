# Finding 2026-10-07, ruled the same day (owner: b, oracle f8175e20): simd.sum is a plain number,
# worked out in full and checked where it lands (simd.sum.lane checks at the lane type, simd.sum.wrap
# wraps there). Four i32 lanes of about 1.2 x 10^9 summed, then mod.floor 1, is 0: the oracle runs
# it; CCA's binary had stopped, as design/simd.md of 2026-09-22 said (seed 30022).
# check claude generator, seed 30022
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append(lend.mut 'buf', value: 45); }
    let mut 'v': u64 = 'm';
    let mut 'p': u64 = 1;
    while 'v' div.floor 'p' at_least 10 {
        change 'p' = 'p' times 10;
    }
    loop {
        std::list.append(lend.mut 'buf', value: to.u8(48 plus ('v' div.floor 'p')));
        change 'v' = 'v' mod.floor 'p';
        if 'p' equal_to 1 { break; }
        change 'p' = 'p' div.floor 10;
    }
    std::list.append(lend.mut 'buf', value: 10);
    no_check write(1, lend 'buf', std::list.length(lend 'buf'));
}

fn put_u ['n': u64] {
    emit('n', false);
}

fn put_i ['n': i64] {
    if 'n' < 0 {
        emit(to.u64(0 minus ('n' plus 1)) plus 1, true);
    } else {
        emit(to.u64('n'), false);
    }
}

enum E1 {
    c1,
    c2: i64,
    c3,
    c4: u64
}

fn b1 [@y, @x] [lent @y 'a': std::list[i8], lent @x 'b': std::list[i8]] -> lent @x std::list[i8] {
    return lend 'b';
}

fn f1 ['p1': u8[min: 1], 'p2': i16] -> u64 {
    let mut 's1': simd[i32, count: 4] = simd.fill(value: ((1073741823 minus to.i32('p1')) shr 10) plus (-25999) times (((to.i32('p1') and.bit (-257893692)) mod.floor 92681) minus 46340));
    change 's1'[pos: 2] = simd.sum('s1') mod.floor 1;
    return 0;
}

MAIN {
    let 'v113': u64 = f1(1, 0);
}
