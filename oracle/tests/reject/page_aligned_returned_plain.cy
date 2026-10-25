# a function handing back a page-aligned list hands back that kind (owner, 2026-10-25)
fn mk [] -> std::list.page_aligned[i32] { let $p: std::list[i32] = [1,]; return give $p; }
MAIN {
    let $a := mk();
}
