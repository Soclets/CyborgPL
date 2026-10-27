# a lend hands over the value where it is, so its sizes must match the parameter's (owner, 2026-09-24: a)
fn total [lent $xs: std::ls[int.sign.32]] -> int.unsign.64:
    ret std::ls.len(lend $xs);
end of fn total
!MAIN:
    let $big: std::ls[int.sign.64] = [1, 2, 3];
    let $n := total(lend $big);
!end of MAIN
