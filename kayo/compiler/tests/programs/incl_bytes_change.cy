# `incl.bytes` is a const's value (owner, 2026-10-10: "(c)"): read in
# place — an element, its length, lent to a call — it is the program's own
# bytes, nothing copied; taken to keep and change (`let mut 'b' := 'four'`)
# it is a fresh array each time. Three passes of 1 + 2 + 3 + 4 with the
# first set to 10 is 57; plus the const's own sum, 10, read in place: 67.
%extern.C proc exit [$code: i32];

const $four := incl.bytes["incl_assets/four.bin"];

fn total [lent $bs: arr[u8]] -> i32:
    var $n: i32 = 0;
    for $x in lend $bs:
        change $n to $n + to.i32($x);
    ;for
    ret $n;
;fn total

MAIN:
    var $sum: i32 = 0;
    for $i [from: 1, to: 3, by: 1]:
        var $b = $four;
        change $b[pos: 1] to $b[pos: 1] + 9;
        change $sum to $sum + total(lend $b);
    ;for
    change $sum to $sum + total(lend $four) - to.i32($four[pos: 4]) + to.i32(arr.len($four));
    no_check my.exit($sum);
;MAIN
