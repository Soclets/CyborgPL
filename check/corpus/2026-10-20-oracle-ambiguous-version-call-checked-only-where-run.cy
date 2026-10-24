# Finding 2026-10-20 (by hand, writing the overloading episode): a call that two versions could take
# is refused at the call (owner, 2026-10-20), but the oracle refuses it only where it runs. With the
# call in a branch never taken, the oracle prints "before" and "after"; kayo refuses it when building:
# "versions of `ov.flip` could take this call: a bare literal fits more than one". Taken, the oracle
# refuses it too, after printing what came before.
# Moved to a `versions` block (owner, 2026-10-21; oracle 78a25324).
coll.fn ov {
    versions ov.flip {
        ['x': i32] -> i32 { return 0 - ('x' mod.floor 50); }
        ['x': i64] -> i32 { return to.i32('x' mod.floor 50); }
    }
}
MAIN {
    let 'p': bool = to.i64(1) equal_to to.i64(2);
    std::print.useful("before", newln);
    if 'p' {
        std::print.useful(ov.flip(77), newln);
    }
    std::print.useful("after", newln);
}
