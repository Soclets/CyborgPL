# The module `shop`, its own file: the root's bounds through `my.`.
pub fn basket [let $count: my.quantity] -> str.utf8:
    ret to.stored.str.utf8($count) concat " in the basket";
end of fn basket

pub fn sample [let $count: my.quantity.small] -> str.utf8:
    ret to.stored.str.utf8($count) concat " to try";
end of fn sample

# The module's own bounds, which the root (or any other file) names by the module's path: `shop.order.discount`.
pub coll.bounded order:
    order.discount = int.sign.32[min: 0, max: 50];
end of coll.bounded order

pub fn off [let $percent: order.discount] -> str.utf8:
    ret to.stored.str.utf8($percent) concat "% off";
end of fn off
