# a root bound named from another file through `my.` checks as the type it names (owner, 2026-10-26)
coll.bounded quantity:
    quantity = i32[min: 1, max: 99];
    quantity.small = i32[min: 1, max: 9];
;coll.bounded quantity

MAIN:
    let $n: quantity := 12;
    std::print(shop.basket($n), " ", shop.sample(10), newln);
;MAIN
