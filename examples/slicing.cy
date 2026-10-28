# list.slice / array.slice (2026-09-23): 1-based and inclusive, like [i] and range[…].
# A slice owns its elements; the prover elides the copy where nobody can tell.
MAIN:
    let $xs := [10, 20, 30, 40, 50];
    let $mid := std::ls.slice(lend $xs, from: pos 2, to: pos 4);
    std::println($mid, " ", std::ls.len(lend $mid));
    let $fixed: arr[i32] := [1, 2, 3];
    std::println(arr.slice(lend $fixed, from: pos 1, to: pos 2));
    let $none := std::ls.slice(lend $xs, from: pos 3, to: pos 2);
    std::println($none, " ", $xs);
;MAIN
