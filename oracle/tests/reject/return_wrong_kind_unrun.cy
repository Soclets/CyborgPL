fn g ['p': uint8] -> uint8 {
    if false { return true; }
    return 1;
}
MAIN {
    let 'x' := g[1];
}
