interface O <T>:
    op lt [$a: <T>, $b: <T>] -> bool;
end of interface O
fn f <T: any and O> [let $a: <T>] -> <T>:
    ret $a;
end of fn f
!MAIN:
!end of MAIN
