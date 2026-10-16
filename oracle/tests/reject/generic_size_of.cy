fn f <T: any> ['a': <T>] {
    let 'n' := size.of[<T>];
}
MAIN {
    f[1];
}
