union.tag S:
    a: int.sign.32;
    b;
end of union.tag S
!MAIN:
    let $s := S.a("x");
!end of MAIN
