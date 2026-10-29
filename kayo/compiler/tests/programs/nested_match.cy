# A `match` alone in a case follows the outer match (owner, 2026-09-29: a):
# in a statement match it runs; in a value match it is the case's value
# (examples/nested_match.cpls as an exit code: "mail" is 2, the price 9 →
# 2 x 10 + 9 = 29).
%extern.C proc exit [let $code: i32];
union.tag Order:
    cancelled; paid;
;union.tag Order
union.tag Ship:
    pickup; post;
;union.tag Ship
MAIN:
    let $o := Order.paid;
    let $s := Ship.post;
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
    let $price := match $o:
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
    no_check my.exit($went x 10 + $price);
;MAIN
