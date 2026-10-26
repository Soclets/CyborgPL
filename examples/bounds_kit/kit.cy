# A kit's root bounds, named from a module in another file through `my.` (owner, 2026-10-26: "yes, no shit"), as
# `my.Color` names a root type. The root names them bare.
coll.bound quantity {
    quantity = i32[min: 1, max: 99];
    quantity.small = i32[min: 1, max: 9];
}

MAIN {
    let $n: quantity = 12;
    std::print.useful(shop.basket($n), " ", shop.sample(3), newln);
}
