# a slice is a list, an array's too (owner, 2026-09-23)
!MAIN:
    let $f: arr[int.sign.32] := [10, 20, 30];
    let $b: arr[int.sign.32] := arr.slice(lend $f, from: pos 1, to: pos 2);
!end of MAIN
