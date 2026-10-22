# Finding 2026-10-20 (by hand, writing the overloading episode): a call with a bool argument, where one
# version takes a bool and the others whole numbers, is refused by kayo as one more than one version
# could take ("versions of `ov` could take this call: say the arguments' types"); only the bool
# version can (owner, 2026-10-20: a version is picked by the arguments' exact types). The oracle picks
# it and prints 1. The i16, i64 and two-argument calls agree.
# Moved to a `versions` block (owner, 2026-10-21; oracle 78a25324).
coll.fn ov {
    versions ov {
        ['x': i32, 'k': i32] -> i32 { return ('x' x 'k') mod.floor 97; }
        ['x': i64] -> i32 { return to.i32['x' mod.floor 1000]; }
        ['x': i16] -> i32 { return to.i32['x'] x 2; }
        ['x': bool] -> i32 { if 'x' { return 1; } return 0; }
    }
}
MAIN {
    std::print.useful[ov[true], " ", ov[false], newln];
}
