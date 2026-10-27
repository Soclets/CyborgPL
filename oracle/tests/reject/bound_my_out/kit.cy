# a root bound named from another file through `my.` checks as the type it names (owner, 2026-10-26)
coll.bounded quantity:
    quantity = int.sign.32[min: 1, max: 99];
    quantity.small = int.sign.32[min: 1, max: 9];
end of coll.bounded quantity

!MAIN:
    let $n: quantity := 12;
    std::print.shortest(shop.basket($n), " ", shop.sample(10), newln);
!end of MAIN
