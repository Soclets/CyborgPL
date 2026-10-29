# In a #strict file a bare nested `match` is a statement, and a nested match
# that is the value says `giveS` (owner, 2026-09-29): "mail" is 2, the price
# 9 → 29 (examples/strict_nested_match.cpls as an exit code).
%extern.C proc exit [$code: i32];
union.tag Order:
    cancelled; paid;
;union.tag Order
union.tag Ship:
    pickup; post;
;union.tag Ship
fn handle [$o: Order, $s: Ship] -> i32:
    var $went: i32 = 0;
    match $o:
        case cancelled:
        ;case cancelled
        case paid:
            match $s:
                case pickup:
                    change $went to 1;
                ;case pickup
                case post:
                    change $went to 2;
                ;case post
            ;match
        ;case paid
    ;match
    ret $went;
;fn handle
fn price [$o: Order, $s: Ship] -> i32:
    let $p: i32 := match $o:
        case cancelled:
            is 0;
        ;case cancelled
        case paid:
            is match $s:
                case pickup:
                    is 5;
                ;case pickup
                case post:
                    is 9;
                ;case post
            ;match;
        ;case paid
    ;match;
    ret $p;
;fn price
MAIN:
    let $w: i32 := handle(Order.paid, Ship.post);
    let $n: i32 := price(Order.paid, Ship.post);
    no_check my.exit($w x 10 + $n);
;MAIN
