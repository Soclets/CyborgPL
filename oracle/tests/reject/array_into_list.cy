# nothing converts by itself: an array into a list is said (owner, 2026-09-23)
function grow [$xs: std::list[integer.signed.32_bit]] -> integer.unsigned.64_bit:
    return std::list.length(lend $xs);
end of function grow
!MAIN:
    let $f: array[integer.signed.32_bit] = [1, 2];
    let $n := grow(give $f);
!end of MAIN
