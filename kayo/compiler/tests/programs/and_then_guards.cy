# `and.then` / `or.else` stop once the answer is known (owner, 2026-09-25), so
# the check on the left guards the read on the right: Precog proves each read
# in range, and the reads that would be out of range never run.
%extern.C proc exit [let $code: i32];

MAIN:
    let $xs := [3, 5, 8];
    var $i: u64 = 0;
    var $hits: i32 = 0;
    var $kept: i32 = 0;
    while $i < 10:
        if $i >= 1 and.then $i <= std::ls.len(lend $xs) and.then $xs[pos: $i] == 5:
            change $hits to $hits + 1;
        ;if
        if $i == 0 or.else $i > std::ls.len(lend $xs) or.else $xs[pos: $i] > 4:
            change $kept to $kept + 1;
        ;if
        change $i to $i + 1;
    ;while
    no_check my.exit(($hits x 100) + $kept);     # 109
;MAIN
