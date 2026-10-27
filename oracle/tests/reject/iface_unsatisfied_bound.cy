sig Ordered <T>:
    op lt [lent $a: <T>, lent $b: <T>] -> bool;
end of sig Ordered
fn max <T: Ordered> [let $a: <T>, let $b: <T>] -> <T>:
    if $b lt $a:
        ret $a;
    end of if
    ret $b;
end of fn max
!MAIN:
    let $x := max("a", "b");
!end of MAIN
