# A kit's root bounds, named from a module in another file through `my.` (owner, 2026-10-26: "yes, no shit"), as
# `my.Color` names a root type. The root names them bare, and names a module's by its path, `shop.order.discount`.
collection.bounded quantity:
    quantity = integer.signed.32_bit[minimum: 1, maximum: 99];
    quantity.small = integer.signed.32_bit[minimum: 1, maximum: 9];
end of collection.bounded quantity

!MAIN:
    let $n: quantity = 12;
    let $d: shop.order.discount = 20;
    std::print.shortest(shop.basket($n), ", ", shop.sample(3), ", ", shop.off($d), newline);
!end of MAIN
