# Two functions calling each other past the call depth: the walk standing
# for the deeper calls walks each one on its own terms, so `od`'s divisor
# keeps the 0 its last call gives, and the check stays (check claude,
# 2026-10-17, corpus ab280a72: proven never zero from {2, 4, 6, 8}).
extern.C fn exit ['code': int32];

fn ev ['n': uint32[min 0, max 23], 'acc': int64] -> int64 {
    if 'n' equal_to 0 { return 'acc'; }
    return od['n' - 1, ('acc' + 20) mod.floor 3710];
}

fn od ['n': uint32[min 0, max 23], 'acc': int64] -> int64 {
    if 'n' equal_to 0 { return 'acc'; }
    return ev['n' - 1, ('acc' x 3) mod.floor ('n' - 1)];
}

MAIN {
    no_check my.exit[to.int32[ev[10, 50] mod.floor 100]];
}
