# The module `shop`, its own file: the root's bounds through `my.`.
pub fn basket [let $count: my.quantity] -> str.utf8:
    ret to.stored.str.utf8($count) ++ " in the basket";
;fn basket

pub fn sample [let $count: my.quantity.small] -> str.utf8:
    ret to.stored.str.utf8($count) ++ " to try";
;fn sample
