# A kit's root bounds, named from a module in another file through `my.` (owner, 2026-10-26: "yes, no shit"), as
# `my.Color` names a root type. The root names them bare, and names a module's by its path, `shop.order.discount`.
coll.bounded quantity:
    quantity = int.sign.32[min: 1, max: 99];
    quantity.small = int.sign.32[min: 1, max: 9];
end of coll.bounded quantity

!MAIN:
    let $n: quantity = 12;
    let $d: shop.order.discount = 20;
    std::print.shortest(shop.basket($n), ", ", shop.sample(3), ", ", shop.off($d), newln);
!end of MAIN
