# Finding 2026-10-20 (check run, 24 programs of a 300-case batch; cut down by hand): a call whose
# argument is a `to.` conversion is refused by kayo as one that several versions could take; the
# conversion says the argument's type (DECISIONS 2026-10-20: "a variable's declared type, a verb on
# one, or a typed literal / `to.` conversion"), so only the i16 version takes ov[to.i16[169]].
# The oracle picks it and prints 676. Typed literals, variables and bare two-argument calls agree.
# Like 2026-10-20-kayo-a-bool-call-beside-whole-number-versions-is-ambiguous.
coll.fn ov {
    ov ['x': i32, 'k': i32] -> i32 { return ('x' x 'k') mod.floor 230; }
    ov ['x': i64] -> i32 { return to.i32['x' mod.floor 230]; }
    ov ['x': i16] -> i32 { return to.i32['x'] x 4; }
}
MAIN {
    std::print.useful[ov[to.i16[169]], newln];
}
