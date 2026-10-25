# `:=` keeps a list's kind (owner, 2026-10-25: a page-aligned list keeps its kind)
MAIN {
    let $a: std::list.page_aligned[i32] = [5, 6];
    let $b := give $a;
    let $c: std::list.not_page_aligned[i32] = give $b;
}
