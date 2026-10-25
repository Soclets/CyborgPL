# one narrowing becomes the other only by a conversion (owner, 2026-10-25)
MAIN {
    let $a: std::list.page_aligned[i32] = [5, 6];
    let $b: std::list.not_page_aligned[i32] = copy $a;
}
