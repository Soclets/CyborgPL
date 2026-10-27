%external.C procedure fill [$b: pointer.mutable[integer.unsigned.8_bit]];
!MAIN:
    let $xs := [1, 2];
    !no_check fill(lend $xs);
!end of MAIN
