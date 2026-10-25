# refuse: error[wrong_type]
# A list narrowed to its memory meets no other list type (design claude, 2026-10-25, until the owner rules how
# they relate): lent to a parameter that takes a plain `std::list[i32]`, it is refused.
fn total [lent $xs: std::list[i32]] -> i32 { return 0; }
MAIN {
    let $xs: std::list.page_aligned[i32] = [1, 2];
    let $t := total(lend $xs);
}
