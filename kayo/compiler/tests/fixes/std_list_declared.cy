# wrong_type: the declared type changed to the value's, std's list written with `std::` outside std.
MAIN:
    let $a: std::ls[i32] := [1, 2];
    let $b: std::ls.page_aligned[i32] := give $a;
;MAIN
