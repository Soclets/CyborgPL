interface Ordered <T>:
    operator less_than [$a: <T>, $b: <T>] -> boolean;
end of interface Ordered
function max <T: Ordered> [$a: <T>, $b: <T>] -> <T>:
    if $b less_than $a:
        return $a;
    end of if
    return $b;
end of function max
!MAIN:
    let $x := max("a", "b");
!end of MAIN
