# The the base, alone: no limbs, no std (owner, 2026-10-08, built 2026-10-09).
# Operators, true-value numbers, `to.` conversions, range types and generics are the base's now;
# Signatar, CyborgPL's base, had none of them. Nothing is printed: printing is std's.

fn biggest <T: any> ['a': <T>, 'b': <T>, 'pick_a': bool] -> <T> {
    if 'pick_a' { return give 'a'; }
    return give 'b';
}

MAIN {
    let 'a': i32 = 2;
    let 'b': i32 = 'a' + 3 x 4;
    let 'small' := to.i8['b'];                     # proven to fit: 14
    let 'level': i32[min 0, max 10] = 7;
    let 'wrapped': u8 = 250 plus.wrap 10;
    let 'bigger' := biggest['b', 'a', 'b' > 'a'];
    let 'same' := 'bigger' equal_to 'b' and not ('small' < 0);
}
