# A generic operator and a specific one for a type it covers: two meanings, an error — the specific
# one does not quietly win (owner, 2026-09-29: a).
sig Number <T>:
    op minus [lent $a: <T>, lent $b: <T>] -> <T>;
    op lt [lent $a: <T>, lent $b: <T>] -> bool;
end of sig Number
op ~= <T: Number> [lent $a: <T>, lent $b: <T>] -> bool:
    ret $a lt $b;
end of op ~=
op ~= [lent $a: int.sign.32, lent $b: int.sign.32] -> bool:
    ret true;
end of op ~=
!MAIN:
!end of MAIN
