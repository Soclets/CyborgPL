# The module `shop`, its own file: the root's bounds through `my.`.
pub fn basket [let $count: my.quantity] -> str.utf8:
    ret to.stored.str.utf8($count) concat " in the basket";
;fn basket

pub fn sample [let $count: my.quantity.small] -> str.utf8:
    ret to.stored.str.utf8($count) concat " to try";
;fn sample
