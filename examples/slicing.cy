# list.slice / array.slice (2026-09-23): 1-based and inclusive, like [i] and range[…].
# A slice owns its elements; the prover elides the copy where nobody can tell.
!MAIN:
    let $xs := [10, 20, 30, 40, 50];
    let $mid := std::ls.slice(lend $xs, from: pos 2, to: pos 4);
    std::print.shortest($mid, " ", std::ls.len(lend $mid), newln);
    let $fixed: arr[int.sign.32] := [1, 2, 3];
    std::print.shortest(arr.slice(lend $fixed, from: pos 1, to: pos 2), newln);
    let $none := std::ls.slice(lend $xs, from: pos 3, to: pos 2);
    std::print.shortest($none, " ", $xs, newln);
!end of MAIN
