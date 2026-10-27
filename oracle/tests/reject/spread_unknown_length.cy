fn sum [...$nums: int.sign.32] -> int.sign.32:
    ret 0;
end of fn sum
!MAIN:
    var $zs := [1, 2];
    std::ls.append(lend.mut $zs, value: 3);      # no bound: length no longer known here
    let $t := sum(...$zs);
!end of MAIN
