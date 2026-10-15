# Iterators (owner, 2026-10-08/09): `fn Counter.next`, called by hand and
# by `for`, taken `lend.mut` (it remains) and `give` (gone after); `break`
# ends an endless one, and `perm` keeps its last item.
extern.C fn exit ['code': int32];
struct Counter {
    now: int32,
    stop: int32
}

fn Counter.next [lent.mut 'c': Counter] -> optional[int32] {
    if 'c'.now equal_to 'c'.stop { return optional.none; }
    change 'c'.now = 'c'.now plus 1;
    return optional.value['c'.now];
}

struct Forever {
    n: int32
}

fn Forever.next [lent.mut 'f': Forever] -> optional[int32] {
    change 'f'.n = 'f'.n plus 1;
    return optional.value['f'.n];
}

MAIN {
    let mut 'sum': int32 = 0;
    let mut 'c' := Counter[now = 0, stop = 5];
    match Counter.next[lend.mut 'c'] {
        value 'x' { change 'sum' = 'sum' plus 'x'; }
        none { change 'sum' = 'sum' plus 1000; }
    }
    for 'x' in lend.mut 'c' {
        if 'x' equal_to 4 { break; }
        change 'sum' = 'sum' plus 10 times 'x';
    }
    for 'x' in give 'c' {
        change 'sum' = 'sum' plus 100 times 'x';
    }
    let mut 'f' := Forever[n = 0];
    for perm 'y' in lend.mut 'f' {
        if 'y' equal_to 3 { break; }
    }
    change 'sum' = 'sum' plus 'y';
    # 1 + 20 + 30 + 500 + 3 = 554 -> exit 554 mod 256 = 42
    no_check my.exit['sum' mod.floor 256];
}
