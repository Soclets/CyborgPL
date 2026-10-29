# What a chip with no OS holds without a heap (design claude, 2026-10-26):
# a fixed array in a variable, changed and printed, lives in the frame; a
# text literal in a variable stays where the program keeps it; a format
# hole is written into the runtime's one scratch buffer. And reaching MAIN's
# end calls the target's `exit` with 0 (owner, 2026-10-26: "yes").
# Lent to the program's own function, which keeps nothing of it: still the
# frame's (check claude, 2026-10-27, `check bare` seed 71).
fn total [lent $xs: arr[i64]] -> i64:
    var $s: i64 = 0;
    for $i: u64 [from: 1, to: arr.len(lend $xs), by: 1]:
        change $s to $s plus.wrap $xs[pos: $i];
    ;for
    ret $s;
;fn total

MAIN:
    var $a: arr[i32] = [5, 3, 9, 1,];
    change $a[pos: 2] to 7;
    std::print($a[pos: 2], " ", $a, newln);
    let $t := "abc";
    std::print($t, " ", 5, newln);
    let $x: u32 := 48879;
    std::print.format("{base: hex, zeros, width: 8 cols} {places: 2}|{width: 5 cols}|{newln}", $x, 2.5, true);
    let $big: arr[i64] := [-90724, 73967, -21168, 47441,];
    let $c: char.byte := `a`;
    std::print.format("{} {}|{width: 3 cols}|{newln}", total(lend $big), $c, $c);
;MAIN
