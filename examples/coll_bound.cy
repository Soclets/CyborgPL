# `coll.bound`: named bounded types (owner, 2026-10-25). Each member names one bounded type, and a use of it is that
# type exactly, so a value moves between the name and the type freely. The collection's name alone is its own bound,
# its members narrow it, and every line writes its full name.
collection.bounded quantity:
    quantity = integer.signed.32_bit[minimum: 1, maximum: 99];
    quantity.small = integer.signed.32_bit[minimum: 1, maximum: 9];
end of collection.bounded quantity

collection.bounded order:
    order.discount = integer.signed.32_bit[minimum: 0, maximum: 50];
    order.name = string.utf8[minimum: 1, maximum: 12 %grapheme_clusters];
    order.few = std::list[integer.signed.32_bit][maximum: 3];
end of collection.bounded order

record Line:
    name: order.name;
    count: quantity;
end of record Line

function apply_discount [$amount: float.%decimal.64_bit, $percent: order.discount] -> float.%decimal.64_bit:
    return $amount minus ($amount times to.float.%decimal.64_bit($percent)) divide.nearest 100;
end of function apply_discount

!MAIN:
    let $line := Line[name = "apple", count = 12];
    let $few: quantity.small = 3;
    let $same: integer.signed.32_bit[minimum: 1, maximum: 99] = $line.count;      # the very type the name stands for
    let $picks: order.few = [4, 8];
    std::print.shortest($line.name, " x", $line.count, ", ", $few, ", ", $same, ", ", $picks, newline);
    std::print.format("80.00 less 25%: {places: 2}{newline}", apply_discount(80.00, 25));
!end of MAIN
