# nothing converts by itself: an array into a list is said (owner, 2026-09-23)
fn grow [$xs: std::ls[i32]] -> u64:
    ret std::ls.len(lend $xs);
;fn grow
MAIN:
    let $f: arr[i32] := [1, 2];
    let $n := grow(give $f);
;MAIN
