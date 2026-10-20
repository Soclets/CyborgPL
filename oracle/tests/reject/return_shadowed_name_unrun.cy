fn g ['p': u8] -> u8 {
    if false {
        shadow 'p': bool = true;
        return 'p';
    }
    return 'p';
}
MAIN {
    let 'x' := g[1];
}
