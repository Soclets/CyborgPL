# A generic operator (owner, 2026-09-29): written once for any type that keeps its interface's
# promises, like a generic fn. An operator may also be written for several types by name; two
# meanings for the same type are an error, a generic one covering a specific one's type included.
sig Number <T>:
    op - [lent $a: <T>, lent $b: <T>] -> <T>;
    op < [lent $a: <T>, lent $b: <T>] -> bool;
;sig Number
op ~= <T: Number> [lent $a: <T>, lent $b: <T>] -> bool:
    if $a < $b:
        ret $b - $a < 10;
    ;if
    ret $a - $b < 10;
;op ~=
rec A:
    var x: i32;
;rec A
rec B:
    var y: i32;
;rec B
op <+> [lent $a: A, lent $b: A] -> i32:
    ret $a.x + $b.x;
;op <+>
op <+> [lent $a: B, lent $b: B] -> i32:
    ret $a.y x $b.y;
;op <+>
MAIN:
    let $m: i32 := 100;
    std::println(95 ~= $m, " ", 50 ~= $m);
    let $p := A[x = 3];
    let $q := B[y = 3];
    std::println($p <+> $p, " ", $q <+> $q);
;MAIN
