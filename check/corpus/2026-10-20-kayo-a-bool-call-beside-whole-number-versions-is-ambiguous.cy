# Finding 2026-10-20 (by hand, writing the overloading episode): a call with a bool argument, where one
# version takes a bool and the others whole numbers, is refused by kayo as one more than one version
# could take ("versions of `ov` could take this call: say the arguments' types"); only the bool
# version can (owner, 2026-10-20: a version is picked by the arguments' exact types). The oracle picks
# it and prints 1. The int16, int64 and two-argument calls agree.
coll.fn ov {
    ov ['x': int32, 'k': int32] -> int32 { return ('x' x 'k') mod.floor 97; }
    ov ['x': int64] -> int32 { return to.int32['x' mod.floor 1000]; }
    ov ['x': int16] -> int32 { return to.int32['x'] x 2; }
    ov ['x': bool] -> int32 { if 'x' { return 1; } return 0; }
}
MAIN {
    std::print.useful[ov[true], " ", ov[false], newln];
}
