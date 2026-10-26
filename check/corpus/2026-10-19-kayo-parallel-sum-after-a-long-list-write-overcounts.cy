# Finding 2026-10-19 (recref --env KAYO_BITS=0, seeds 228 and 278, then the batch; cut down by hand): a
# loop over a list, summing it, after a loop that wrote the list at more than about 4000 places, adds
# far too much in kayo. Here 8000 ones sum to 8000 in the oracle; kayo prints 44000 (and 11479, 21120,
# 48683 for list[bool] counts of 4538, 8000, 20000 trues, changing from build to build). Fine: a write
# loop of 4000 or 4100; the sum alone (no write loop before it, or one change); the sum written `for
# seq`. Wrong still: the write loop written `for seq`, a list[u8] as well as a list[bool], with
# KAYO_BITS=0 too. Not bit packing; the parallel reduction after a write loop.
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
MAIN {
    let mut 'b': std::list[u8] = std::list.fill(value: 0, count: 8000);
    for 'i': u64 [from: 1, to: 8000, by: 1] { change 'b'[pos: 'i'] = 1; }
    let mut 'c': u64 = 0;
    for 'i': u64 [from: 1, to: std::list.length(lend 'b'), by: 1] { change 'c' = 'c' + to.u64('b'[pos: 'i']); }
    put_u('c');
}
