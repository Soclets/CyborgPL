interface O <T>:
    operator less_than [$a: <T>, $b: <T>] -> boolean;
end of interface O
function f <T: any and O> [$a: <T>] -> <T>:
    return $a;
end of function f
!MAIN:
!end of MAIN
