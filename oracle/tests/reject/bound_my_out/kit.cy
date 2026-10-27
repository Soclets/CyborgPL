# a root bound named from another file through `my.` checks as the type it names (owner, 2026-10-26)
collection.bounded quantity:
    quantity = integer.signed.32_bit[minimum: 1, maximum: 99];
    quantity.small = integer.signed.32_bit[minimum: 1, maximum: 9];
end of collection.bounded quantity

!MAIN:
    let $n: quantity = 12;
    std::print.shortest(shop.basket($n), " ", shop.sample(10), newline);
!end of MAIN
