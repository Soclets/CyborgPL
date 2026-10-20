# a function carrying a copy/give capture is owned: copying it takes a word (owner, 2026-09-24: a)
MAIN {
    let 'xs' := [1, 2];
    let 'f' := fn [] -> u64 with [copy 'xs'] { return list.length[lend 'xs']; };
    let 'g' := 'f';
}
