# Inside `for 'x' in lend.mut 'xs'`, the list may be read once 'x' is used for the last time in
# that trip, and the read sees the new values (owner, 2026-09-30: a, kayo's liveness rule).
MAIN {
    let mut 'xs': list[int32] = [1, 2, 3];
    let mut 'seen': list[int32] = [];
    for 'x' in lend.mut 'xs' {
        change 'x' = 'x' x 10;
        list.append[lend.mut 'seen', value 'xs'[pos 1]];
    }
    std::print.stored['xs', newln];
    std::print.stored['seen', newln];
}
