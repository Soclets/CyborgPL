rec Pair [<A>: any, <B>: any]:
    var left: <A>;
    var right: <B>;
;rec Pair
MAIN:
    let $p: Pair[i32] := Pair[left = 1, right = 2];
;MAIN
