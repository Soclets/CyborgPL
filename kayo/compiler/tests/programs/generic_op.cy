# A generic operator (owner, 2026-09-29), and one operator for two types by
# name (examples/generic_op.cpls as an exit code): 95 ~= 100 is true (1), 50
# ~= 100 false, A 3 <+> A 3 is 6 and B 3 <+> B 3 is 9 → 1 + 60 + 9 x 100 = 961,
# which exits as 961 mod 256 = 193.
%extern.C proc exit [let $code: i32];
sig Number [<T>]:
    op - [lent $a: <T>, lent $b: <T>] -> <T>;
    op < [lent $a: <T>, lent $b: <T>] -> bool;
;sig Number
op ~= [<T>: Number] [lent $a: <T>, lent $b: <T>] -> bool:
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
    var $k: i32 = 0;
    if 95 ~= $m:
        change $k to 1;
    ;if
    if 50 ~= $m:
        change $k to $k + 2;
    ;if
    let $p := A[x = 3];
    let $q := B[y = 3];
    no_check my.exit($k + ($p <+> $p) x 10 + ($q <+> $q) x 100);
;MAIN
