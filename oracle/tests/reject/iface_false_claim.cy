sig Ordered <T>:
    op lt [$a: <T>, $b: <T>] -> bool;
end of sig Ordered
rec V:
    var x: int.sign.32;
end of rec V
V satisfies Ordered;
!MAIN:
!end of MAIN
