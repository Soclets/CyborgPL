# two meanings of one operator for the same types (2026-09-23)
rec A:
    var x: int.sign.32;
end of rec A
op <+> [lent $a: A, lent $b: A] -> int.sign.32:
    ret 1;
end of op <+>
op <+> [lent $a: A, lent $b: A] -> int.sign.32:
    ret 2;
end of op <+>
!MAIN:
!end of MAIN
