%extern.C proc qsort [let $base: ptr.mut[int.sign.32], let $n: int.unsign.64, let $size: int.unsign.64, let $cmp: fn[ptr[int.sign.32], ptr[int.sign.32]] -> int.sign.32];
fn one [let $a: ptr[int.sign.32]] -> int.sign.32:
    ret 0;
end of fn one
!MAIN:
    var $nums := [3, 1, 2];
    !no_check qsort(lend.mut $nums, 3, 4, one);
!end of MAIN
