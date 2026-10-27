interface O <T>:
    op less_than [$a: <T>, $b: <T>] -> bool;
end of interface O
fn f <T: any and O> [$a: <T>] -> <T>:
    ret $a;
end of fn f
!MAIN:
!end of MAIN
