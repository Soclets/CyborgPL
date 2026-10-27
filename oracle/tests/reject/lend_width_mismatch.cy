# a lend hands over the value where it is, so its sizes must match the parameter's (owner, 2026-09-24: a)
function total [lent $xs: std::list[integer.signed.32_bit]] -> integer.unsigned.64_bit:
    return std::list.length(lend $xs);
end of function total
!MAIN:
    let $big: std::list[integer.signed.64_bit] = [1, 2, 3];
    let $n := total(lend $big);
!end of MAIN
