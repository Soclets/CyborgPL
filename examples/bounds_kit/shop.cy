# The module `shop`, its own file: the root's bounds through `my.`.
pub fn basket [$count: my.quantity] -> str.utf8:
    ret to.stored.str.utf8($count) concat " in the basket";
;fn basket

pub fn sample [$count: my.quantity.small] -> str.utf8:
    ret to.stored.str.utf8($count) concat " to try";
;fn sample

# The module's own bounds, which the root (or any other file) names by the module's path: `shop.order.discount`.
pub coll.bounded order:
    order.discount = i32[min: 0, max: 50];
;coll.bounded order

pub fn off [$percent: order.discount] -> str.utf8:
    ret to.stored.str.utf8($percent) concat "% off";
;fn off
