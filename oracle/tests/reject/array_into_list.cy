# nothing converts by itself: an array into a list is said (owner, 2026-09-23)
fn grow [let $xs: std::ls[int.sign.32]] -> int.unsign.64:
    ret std::ls.len(lend $xs);
end of fn grow
!MAIN:
    let $f: arr[int.sign.32] := [1, 2];
    let $n := grow(give $f);
!end of MAIN
