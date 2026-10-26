# A `coll.bound` name is a type wherever it starts a path, and a field of the same name stays a field: declared
# (`quantity: quantity`), built (`quantity = 4`) and read (`$o.quantity`). 4 + 5 = 9: exit 9.
coll.bound quantity {
    quantity = i32[min: 1, max: 99];
    quantity.small = i32[min: 1, max: 9];
}
struct Order { quantity: quantity, spare: quantity.small }
fn total [lent $o: Order] -> i32 { return $o.quantity + $o.spare; }
MAIN {
    let $o := Order[quantity = 4, spare = 5];
    std::exit(code: total(lend $o));
}
