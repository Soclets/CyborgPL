%extern.C proc qsort [$base: ptr.mut[int.sign.32], $n: int.unsign.64, $size: int.unsign.64, $cmp: fn[ptr[int.sign.32], ptr[int.sign.32]] -> int.sign.32];
fn one [$a: ptr[int.sign.32]] -> int.sign.32:
    ret 0;
end of fn one
!MAIN:
    var $nums := [3, 1, 2];
    !no_check qsort(lend.mut $nums, 3, 4, one);
!end of MAIN
