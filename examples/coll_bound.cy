# `coll.bound`: named bounded types (owner, 2026-10-25). Each member names one bounded type, and a use of it is that
# type exactly, so a value moves between the name and the type freely. The collection's name alone is its own bound,
# its members narrow it, and every line writes its full name.
coll.bounded quantity:
    quantity = i32[min: 1, max: 99];
    quantity.small = i32[min: 1, max: 9];
;coll.bounded quantity

coll.bounded order:
    order.discount = i32[min: 0, max: 50];
    order.name = str.utf8[min: 1, max: 12 %clusters];
    order.few = std::ls[i32][max: 3];
;coll.bounded order

rec Line:
    var name: order.name;
    var count: quantity;
;rec Line

fn apply_discount [let $amount: %d64, let $percent: order.discount] -> %d64:
    ret $amount - ($amount x to.%d64($percent)) div.near 100;
;fn apply_discount

MAIN:
    let $line := Line[name = "apple", count = 12];
    let $few: quantity.small := 3;
    let $same: i32[min: 1, max: 99] := $line.count;      # the very type the name stands for
    let $picks: order.few := [4, 8];
    std::print($line.name, " x", $line.count, ", ", $few, ", ", $same, ", ", $picks, newln);
    std::print.format("80.00 less 25%: {places: 2}{newln}", apply_discount(80.00, 25));
;MAIN
