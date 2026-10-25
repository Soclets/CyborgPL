# a plain list becomes page-aligned only by a conversion (owner, 2026-10-25)
MAIN {
    let $plain: std::list[i32] = [5, 6];
    let $pa: std::list.page_aligned[i32] = give $plain;
}
