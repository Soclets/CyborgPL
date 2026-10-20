# Iterators (owner, 2026-10-08/09): a type with its own `next` returning `optional[…]` is walked by
# `for`. `fn Counter.next` belongs to `Counter` and is called `Counter.next[…]` ("`b`"); `for` takes
# the iterator with `lend.mut` (it remains, used up as far as the loop went) or `give` (gone after)
# ("`c`"). `optional` is the base's: `optional.value[…]` / `optional.none`, matched bare.

struct Counter {
    now: i32,
    stop: i32
}

fn Counter.next [lent.mut 'c': Counter] -> optional[i32] {
    if 'c'.now equal_to 'c'.stop { return optional.none; }
    change 'c'.now = 'c'.now + 1;
    return optional.value['c'.now];
}

struct Forever {
    n: i32
}

fn Forever.next [lent.mut 'f': Forever] -> optional[i32] {
    change 'f'.n = 'f'.n + 1;
    return optional.value['f'.n];
}

MAIN {
    let mut 'c' := Counter[now = 0, stop = 5];
    match Counter.next[lend.mut 'c'] {
        value 'x' { std::print.useful["by hand: ", 'x', newln]; }
        none { std::print.useful["empty", newln]; }
    }
    for 'x' in lend.mut 'c' {
        if 'x' equal_to 4 { break; }
        std::print.useful["lent: ", 'x', newln];
    }
    for 'x' in give 'c' {
        std::print.useful["given: ", 'x', newln];
    }
    let mut 'f' := Forever[n = 0];
    for perm 'y' in lend.mut 'f' {
        if 'y' equal_to 3 { break; }
    }
    std::print.useful["stopped at ", 'y', newln];
}
