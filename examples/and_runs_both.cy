# `and` and `or` always work out both sides, left then right: nothing written on the
# line is skipped (owner, 2026-09-25: b). A check that protects the next step is
# therefore written as two `if`s; tests/reject/and_does_not_guard.cy is the one-line
# form, which stops when the right side is out of range.
fn loud ['name': str.utf8, 'answer': bool] -> bool {
    std::print.useful["checked ", 'name', newln];
    return 'answer';
}
MAIN {
    let 'ok' := loud["first", false] and loud["second", true];    # both print
    std::print.useful['ok', newln];
    let 'xs' := [3, 5, 8];
    let 'i': u64 = 7;
    if 'i' at_most std::list.length[lend 'xs'] {
        if 'xs'[pos: 'i'] equal_to 5 { std::print.useful["found", newln]; }
    }
    std::print.useful["done", newln];
}
