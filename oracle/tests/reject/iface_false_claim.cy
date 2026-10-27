interface Ordered <T>:
    operator less_than [$a: <T>, $b: <T>] -> boolean;
end of interface Ordered
record V:
    x: integer.signed.32_bit;
end of record V
V satisfies Ordered;
!MAIN:
!end of MAIN
