fn half ['n': i32] -> i32 or fail[odd, negative: i32] {
    if 'n' < 0 { fail negative['n']; }
    return 'n';
}
coll.handler h {
    h.zero { odd { return 0; } }
}
fn f [] -> i32 {
    let 'v' := half[3] otherwise h.zero;
    return 'v';
}
MAIN { std::print.useful[f[], newln]; }
