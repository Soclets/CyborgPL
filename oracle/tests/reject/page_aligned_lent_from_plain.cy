# a lend into a page-aligned parameter must be of that kind (owner, 2026-10-25)
fn f [lent.mut $xs: std::list.page_aligned[i32]] { }
MAIN {
    let mut $plain: std::list[i32] = [5, 6];
    f(lend.mut $plain);
}
