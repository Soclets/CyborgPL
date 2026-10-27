# A generic operator and a specific one for a type it covers: two meanings, an error — the specific
# one does not quietly win (owner, 2026-09-29: a).
interface Number <T>:
    op minus [$a: <T>, $b: <T>] -> <T>;
    op less_than [$a: <T>, $b: <T>] -> bool;
end of interface Number
op ~= <T: Number> [$a: <T>, $b: <T>] -> bool:
    ret $a less_than $b;
end of op ~=
op ~= [$a: int.sign.32, $b: int.sign.32] -> bool:
    ret true;
end of op ~=
!MAIN:
!end of MAIN
