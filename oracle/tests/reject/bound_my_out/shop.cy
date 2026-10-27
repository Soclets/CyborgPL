# The module `shop`, its own file: the root's bounds through `my.`.
public function basket [$count: my.quantity] -> string.utf8:
    return to.stored.string.utf8($count) concatenate " in the basket";
end of function basket

public function sample [$count: my.quantity.small] -> string.utf8:
    return to.stored.string.utf8($count) concatenate " to try";
end of function sample
