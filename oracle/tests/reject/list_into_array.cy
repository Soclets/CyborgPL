# nothing converts by itself: a list into an array is said (owner, 2026-09-23)
MAIN:
    let $l: std::ls[i32] := [1, 2];
    let $a: arr[i32] := give $l;
;MAIN
