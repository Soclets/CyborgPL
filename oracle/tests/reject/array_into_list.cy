# nothing converts by itself: an array into a list is said (owner, 2026-09-23)
fn grow [$xs: std::list[i32]] -> u64 { return std::list.length(lend $xs); }
MAIN {
    let $f: array[i32] = [1, 2];
    let $n := grow(give $f);
}
