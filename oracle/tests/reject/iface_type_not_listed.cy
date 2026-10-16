interface Number {
    int32
    bin64
}
fn f <T: Number> ['a': <T>] -> <T> { return 'a'; }
MAIN {
    let 's' := f["text"];
}
