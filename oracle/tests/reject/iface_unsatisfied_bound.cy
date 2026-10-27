interface Ordered <T>:
    op less_than [$a: <T>, $b: <T>] -> bool;
end of interface Ordered
fn max <T: Ordered> [let $a: <T>, let $b: <T>] -> <T>:
    if $b less_than $a:
        ret $a;
    end of if
    ret $b;
end of fn max
!MAIN:
    let $x := max("a", "b");
!end of MAIN
