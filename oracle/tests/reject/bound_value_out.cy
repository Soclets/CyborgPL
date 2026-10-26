# a named bound checks as the type it names
coll.bound quantity {
    quantity = i32[min: 1, max: 99];
}
MAIN {
    let $n: quantity = 100;
}
