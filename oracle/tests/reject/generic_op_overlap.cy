# A generic operator and a specific one for a type it covers: two meanings, an error — the specific
# one does not quietly win (owner, 2026-09-29: a).
sig Number <T>:
    op - [lent $a: <T>, lent $b: <T>] -> <T>;
    op < [lent $a: <T>, lent $b: <T>] -> bool;
;sig Number
op ~= <T: Number> [lent $a: <T>, lent $b: <T>] -> bool:
    ret $a < $b;
;op ~=
op ~= [lent $a: i32, lent $b: i32] -> bool:
    ret true;
;op ~=
MAIN:
;MAIN
