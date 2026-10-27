# an array taken over by `:=` is still an array, so it cannot land in a list's place (2026-09-23)
!MAIN:
    let $a: arr[int.sign.32] = [5, 6];
    let $b := give $a;
    let $c: std::ls[int.sign.32] = give $b;
!end of MAIN
