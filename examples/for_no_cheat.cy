# `no_cheat` — "no cheat" (owner, 2026-10-01): the work written really happens, so the compiler may not
# replace it with its answer (here n(n+1)/2), drop it, or compute look-alike calls once; SIMD, cores and
# inlining are still allowed. It goes in front of any statement or block, as `no_check` does, calls
# included (owner, 2026-10-18: "(b)"), and in front of a `for`, never inside it (owner: "(a)").
fn fib ['n': int64[min 0, max 92]] -> int64 {
    if 'n' < 2 { return 'n'; }
    return fib['n' - 1] + fib['n' - 2];
}

MAIN {
    let mut 'acc': int64 = 0;
    no_cheat for 'i': int64 from 1 to 1000 by 1 {
        change 'acc' = 'acc' + 'i';
    }
    let mut 'more': int64 = 0;
    no_cheat for seq 'i': int64 from 1 to 10 by 1 {
        change 'more' = 'more' + 'i';
    }
    no_cheat let 'f' := fib[20];
    let mut 'both': int64 = 0;
    no_cheat {
        change 'both' = 'acc' + 'more';
    }
    std::print.useful.auto_newln['acc', " ", 'more', " ", 'f', " ", 'both'];
}
