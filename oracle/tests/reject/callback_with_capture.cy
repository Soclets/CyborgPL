%extern.C proc qsort [let $base: ptr.mut[int.sign.32], let $n: int.unsign.64, let $size: int.unsign.64, let $cmp: fn[ptr[int.sign.32], ptr[int.sign.32]] -> int.sign.32];
!MAIN:
    var $nums = [3, 1, 2];
    let $flip := true;
    !no_check qsort(lend.mut $nums, 3, 4, fn [$a: ptr[int.sign.32], $b: ptr[int.sign.32]] -> int.sign.32 with [lend $flip]:
        ret 0;
    end of fn);
!end of MAIN
