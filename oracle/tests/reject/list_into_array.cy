# nothing converts by itself: a list into an array is said (owner, 2026-09-23)
!MAIN:
    let $l: std::ls[int.sign.32] := [1, 2];
    let $a: arr[int.sign.32] := give $l;
!end of MAIN
