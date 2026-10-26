# Finding 2026-10-17 (check run, seeds 2050 and 2058, cut down by hand): Precog's layer-2 value set for
# a mod's divisor in mutual recursion keeps four values and drops the rest, then proves the divisor is
# never zero. ev[10, 50] reaches od with 'n' 9, 7, 5, 3, 1, so ('acc' x 3) mod.floor ('n' - 1) is a mod
# by zero at the last: the oracle stops there. Precog claims the divisor's values are {2, 4, 6, 8} and
# proves "the divisor is never zero"; the check is gone and kayo prints 60. From ev[8] the set is
# {0, 2, 4, 6}, the site is checked and kayo stops too. Not the recursion inlining (0ed08993), which
# leaves mutual recursion alone.
# check claude generator (printer only)
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
fn ev ['n': u32[min: 0, max: 23], 'acc': i64] -> i64 {
    if 'n' equal_to 0 { return 'acc'; }
    put_i('acc');
    return od('n' - 1, ('acc' + 20) mod.floor 3710);
}
fn od ['n': u32[min: 0, max: 23], 'acc': i64] -> i64 {
    if 'n' equal_to 0 { return 'acc'; }
    return ev('n' - 1, ('acc' x 3) mod.floor ('n' - 1));
}
MAIN {
    put_i(ev(10, 50));
}
