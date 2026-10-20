interface Number {
    i32
    f64
}
fn f <T: Number> ['a': <T>] -> <T> { return 'a'; }
MAIN {
    let 's' := f["text"];
}
