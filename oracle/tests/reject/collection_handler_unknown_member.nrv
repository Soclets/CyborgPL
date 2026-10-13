fn half ['n': int32] -> int32 or fail[odd, negative: int32] {
    if 'n' < 0 { fail negative['n']; }
    return 'n';
}
collection.handler h {
    zero { odd { return 0; } negative 'v' { return 0; } }
}
fn f [] -> int32 {
    let 'v' := half[3] otherwise h.nope;
    return 'v';
}
MAIN { }
