# The module `shop`, its own file: the root's bounds through `my.`.
public function basket [$count: my.quantity] -> string.utf8:
    return to.stored.string.utf8($count) concatenate " in the basket";
end of function basket

public function sample [$count: my.quantity.small] -> string.utf8:
    return to.stored.string.utf8($count) concatenate " to try";
end of function sample

# The module's own bounds, which the root (or any other file) names by the module's path: `shop.order.discount`.
public collection.bounded order:
    order.discount = integer.signed.32_bit[minimum: 0, maximum: 50];
end of collection.bounded order

public function off [$percent: order.discount] -> string.utf8:
    return to.stored.string.utf8($percent) concatenate "% off";
end of function off
