# Finding 2026-10-16 (a width probe, by: hand), ruled 2026-10-17 (owner: "(a)"; oracle 915c0426,
# DECISIONS 4478729d): a shift's count gives nothing inside it a width, so the bare 5 shifted in
# to.i64[1] shl (5 shr to.u8[1]) has none: "`shr` needs a width", refused by both engines,
# whether or not the code runs. Before the ruling the oracle ran it (count 2, printing 4).
# check claude generator (printer only)
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

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
MAIN {
    let 'r': i64 = to.i64(1) shl (5 shr to.u8(1));
    put_i('r');
    put_u(7);
}
