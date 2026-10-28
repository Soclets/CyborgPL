# Inside `for $x in lend.mut $xs`, the list may be read once 'x' is used for the last time in
# that trip, and the read sees the new values (owner, 2026-09-30: a, kayo's liveness rule).
MAIN:
    var $xs: std::ls[i32] = [1, 2, 3];
    var $seen: std::ls[i32] = [];
    for $x in lend.mut $xs:
        change $x x= 10;
        std::ls.append(lend.mut $seen, value: $xs[pos: 1]);
    ;for
    std::println.stored($xs);
    std::println.stored($seen);
;MAIN
