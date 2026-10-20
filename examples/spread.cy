# `...'xs'` at a call: unpack a list into the variadic part (decided 2026-09-21).
# Only where the list's length is known — a literal, a `list[T][lo, hi]`, a bundle.

fn sum [...'nums': i32] -> i32 {
    let mut 'total' := 0;
    for 'n' in lend 'nums' { change 'total' = 'total' + 'n'; }
    return 'total';
}

fn show [lent 'label': str.utf8, lent ...'parts'] {
    std::print.useful['label', ":"];
    for 'p' in lend 'parts' { std::print.useful[" ", 'p']; }
    std::print.stored[newln];
}

fn shout [lent ...'parts'] { show["loud", ...'parts']; }      # a bundle passed on: its count is known

fn keep [...'names': str.utf8] -> u64 { return list.length[lend 'names']; }   # takes ownership of each

MAIN {
    let 'xs' := [1, 2, 3];
    std::print.useful[sum[...'xs'], newln];                   # same as sum[1, 2, 3]
    std::print.useful[sum[10, ...'xs', 20], newln];           # mixed with hand-typed values
    let mut 'ys': list[i32][min: 0, max: 4] = list.empty;
    list.append[lend.mut 'ys', value: 7];
    list.append[lend.mut 'ys', value: 8];
    std::print.useful[sum[...'ys'], newln];                   # bounded: at most four
    let 'words' := ["a", "b"];
    show["letters", ...'words'];                            # no verb: the elements are lent
    shout[1, true];
    std::print.useful[keep[...copy 'words'], " kept, ", 'words', " still here", newln];
    std::print.useful[keep[...give 'words'], " kept, ", 'words', " left", newln];
}
