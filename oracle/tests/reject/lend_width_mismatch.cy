# a lend hands over the value where it is, so its sizes must match the parameter's (owner, 2026-09-24: a)
fn total [lent $xs: std::ls[i32]] -> u64:
    ret std::ls.len(lend $xs);
;fn total
MAIN:
    let $big: std::ls[i64] := [1, 2, 3];
    let $n := total(lend $big);
;MAIN
