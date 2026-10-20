# list.slice / array.slice (2026-09-23): 1-based and inclusive, like [i] and range[…].
# A slice owns its elements; the prover elides the copy where nobody can tell.
MAIN {
    let 'xs' := [10, 20, 30, 40, 50];
    let 'mid' := list.slice[lend 'xs', from pos 2, to pos 4];
    std::print.useful['mid', " ", list.length[lend 'mid'], newln];
    let 'fixed': array[i32] = [1, 2, 3];
    std::print.useful[array.slice[lend 'fixed', from pos 1, to pos 2], newln];
    let 'none' := list.slice[lend 'xs', from pos 3, to pos 2];
    std::print.useful['none', " ", 'xs', newln];
}
