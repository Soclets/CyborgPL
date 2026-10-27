# The module `shop`, its own file: the root's bounds through `my.`.
pub fn basket [$count: my.quantity] -> str.utf8:
    ret to.stored.str.utf8($count) concat " in the basket";
end of fn basket

pub fn sample [$count: my.quantity.small] -> str.utf8:
    ret to.stored.str.utf8($count) concat " to try";
end of fn sample
