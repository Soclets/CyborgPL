record Pair <A: any, B: any>:
    left: <A>;
    right: <B>;
end of record Pair
!MAIN:
    let $p: Pair[integer.signed.32_bit] = Pair[left = 1, right = 2];
!end of MAIN
