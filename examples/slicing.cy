# list.slice / array.slice (2026-09-23): 1-based and inclusive, like [i] and range[…].
# A slice owns its elements; the prover elides the copy where nobody can tell.
!MAIN:
    let $xs := [10, 20, 30, 40, 50];
    let $mid := std::list.slice(lend $xs, from: position 2, to: position 4);
    std::print.shortest($mid, " ", std::list.length(lend $mid), newline);
    let $fixed: array[integer.signed.32_bit] = [1, 2, 3];
    std::print.shortest(array.slice(lend $fixed, from: position 1, to: position 2), newline);
    let $none := std::list.slice(lend $xs, from: position 3, to: position 2);
    std::print.shortest($none, " ", $xs, newline);
!end of MAIN
