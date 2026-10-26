# Question 2026-09-30 (check run, seed 9621, shrunk): inside for 'x26' in lend.mut 'xs25', the body reads
# 'xs25'['i30'] right after change 'x26' = 'x26' + 1. CCA reads the new value (15; the element is the
# list's own, 4ef0ed8); the oracle the old one (14) until the trip ends. Rust-style borrowing would
# refuse the read while 'x26' borrows the list mutably; CCA refused it in seeds 9512, 9542, 9663 and
# ran it here. Whether it is allowed is with the owner.
# check claude generator, seed 9621
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
    if 'n' < 0 {
        emit(to.u64(0 minus ('n' plus 1)) plus 1, true);
    } else {
        emit(to.u64('n'), false);
    }
}

MAIN {
    let 'v22': u8 = 1;
    for 'i24': i8 [from: 120, to: 127, by: 1] {
        let mut 'xs25': std::list[u8] = std::list.fill(value: ((('v22' plus 0) mod.floor (((('v22' mod.floor 4) pow (to.u8('i24') mod.floor 6)) mod.floor 122) plus 1)) mod.floor 110) plus 13, count: ((-2147483647) minus (-190277311)) mod.floor 9);
        for 'x26' in lend.mut 'xs25' {
            change 'x26' = 'x26' plus 1;
            let 'v31': i64 = 0;
            for 'i30': u64 [from: 1, to: 1, by: 1] {
                change 'v31' = 'xs25'[pos: 'i30'];
            }
            put_i('v31');
        }
    }
}
