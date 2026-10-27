sig O <T>:
    op lt [lent $a: <T>, lent $b: <T>] -> bool;
end of sig O
fn f <T: any and O> [let $a: <T>] -> <T>:
    ret $a;
end of fn f
!MAIN:
!end of MAIN
