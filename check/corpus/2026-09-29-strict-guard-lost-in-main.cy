# Finding 2026-09-29 (check run, seed 6430, shrunk): in a #strict file's MAIN, 'v16' !== 0 and.then
# (15 mod 'v16') is refused ("the divisor can be zero"); 'v16' is 1 from f1[1]. 7ef4341 stopped folds
# inside strict functions; MAIN seems to still fold the guard away. The oracle runs it.
# check claude generator, seed 6430
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append(lend.mut 'buf', value: 45); }
    let mut 'started': bool = false;
    let mut 'p': u64 = 10000000000000000000;
    for temp 'k': i32 [from: 1, to: 20, by: 1] {
        if 'p' > 0 {
            let immut 'd': u64 = ('m' div.floor 'p') mod.floor 10;
            if 'd' not_equal 0 or 'started' or 'p' equal_to 1 {
                change 'started' = true;
                std::list.append(lend.mut 'buf', value: to.u8(48 plus 'd'));
            }
            change 'p' = 'p' div.floor 10;
        }
    }
    std::list.append(lend.mut 'buf', value: 10);
    no_check write(1, lend 'buf', std::list.length(lend 'buf'));
}

fn put_u ['n': u64] {
    emit('n', false);
}

fn put_i ['n': i64] {
    if 'n' at_least 0 {
        emit(to.u64('n'), false);
    } else {
        let immut 'up': i64 = -('n' plus 1);
        emit(to.u64('up') plus 1, true);
    }
}

fn f1 ['p1': u16] -> u32 {
    let immut 'v1': u8 = (to.u8('p1' mod.floor 256) mod.floor 16) times (to.u8('p1' mod.floor 256) mod.floor 16);
    return to.u32('v1');
}

fn f2 ['p1': u16] -> i32 {
    return 2147483646;
}

MAIN {
    let 'v16': u32 = f1(1) plus 0;
    let 'v17': i64 = 19 minus to.i64('v16');
    for temp 'i19': u8 [from: 33, to: 42, by: 1] {
        let mut 'w22': u64 = 3;
        loop {
            if 'w22' at_most 0 {
                break;
            }
            change 'w22' = 'w22' minus 1;
        }
    }
    for temp 'i24': i8 [from: 35, to: 31, by: 1] {
    }
    if ('v16' not_equal 0 and.then (15 mod.floor 'v16') not_equal 3) or 16 div.floor ((to.u64('v16') mod.floor 9159883580958452947) plus 1) > 0 {
    } else if to.i32('v16' mod.floor 2147483648) equal_to ((to.i32('v16' mod.floor 2147483648) mod.floor 92681) minus 46340) times ((f2(to.u16('v17' mod.floor 65536)) mod.floor 92681) minus 46340) {
        for temp 'i37': u16 [from: 31, to: 34, by: 1] {
        }
    } else {
        let mut 'w42': u32 = 9;
        loop {
            if 'w42' at_most 0 {
                break;
            }
            change 'w42' = 'w42' minus 1;
        }
    }
}
