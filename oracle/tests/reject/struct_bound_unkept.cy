interface Number {
    i32
}
struct Box <T: Number> { item: <T> }
MAIN {
    let 'b': Box[str.utf8] = Box[item = "x"];
}
