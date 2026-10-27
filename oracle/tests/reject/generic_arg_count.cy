rec Pair <A: any, B: any>:
    var left: <A>;
    var right: <B>;
end of rec Pair
!MAIN:
    let $p: Pair[int.sign.32] := Pair[left = 1, right = 2];
!end of MAIN
