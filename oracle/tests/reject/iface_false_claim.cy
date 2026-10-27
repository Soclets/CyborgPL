interface Ordered <T>:
    op less_than [$a: <T>, $b: <T>] -> bool;
end of interface Ordered
rec V:
    var x: int.sign.32;
end of rec V
V satisfies Ordered;
!MAIN:
!end of MAIN
