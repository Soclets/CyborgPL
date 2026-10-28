# check claude's borrow findings (2026-09-29): a borrow never aliases a change and never outlives its owner (core-features.md)
MAIN:
    var $s: std::ls[i32] = [1, 2, 3];
    for $x in lend $s:
        if $x == 1:
            std::ls.append(lend.mut $s, value: 5);
        ;if
    ;for
;MAIN
