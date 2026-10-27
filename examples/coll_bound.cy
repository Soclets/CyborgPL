# `coll.bound`: named bounded types (owner, 2026-10-25). Each member names one bounded type, and a use of it is that
# type exactly, so a value moves between the name and the type freely. The collection's name alone is its own bound,
# its members narrow it, and every line writes its full name.
coll.bounded quantity:
    quantity = int.sign.32[min: 1, max: 99];
    quantity.small = int.sign.32[min: 1, max: 9];
end of coll.bounded quantity

coll.bounded order:
    order.discount = int.sign.32[min: 0, max: 50];
    order.name = str.utf8[min: 1, max: 12 %clusters];
    order.few = std::ls[int.sign.32][max: 3];
end of coll.bounded order

rec Line:
    var name: order.name;
    var count: quantity;
end of rec Line

fn apply_discount [let $amount: float.%deci.64, let $percent: order.discount] -> float.%deci.64:
    ret $amount minus ($amount times to.float.%deci.64($percent)) div.near 100;
end of fn apply_discount

!MAIN:
    let $line := Line[name = "apple", count = 12];
    let $few: quantity.small := 3;
    let $same: int.sign.32[min: 1, max: 99] := $line.count;      # the very type the name stands for
    let $picks: order.few := [4, 8];
    std::print.shortest($line.name, " x", $line.count, ", ", $few, ", ", $same, ", ", $picks, newln);
    std::print.format("80.00 less 25%: {places: 2}{newln}", apply_discount(80.00, 25));
!end of MAIN
