# A `match` alone in a case is a statement (owner, 2026-09-29); a nested match that is the case's value
# says `is match …` (owner, 2026-10-07: a case says its value with `is`).
union.tag Order:
    cancelled;
    paid;
;union.tag Order
union.tag Ship:
    pickup;
    post;
;union.tag Ship
MAIN:
    let $o := Order.paid;
    let $s := Ship.post;
    match $o:
        case cancelled:
        ;case cancelled
        case paid:
            match $s:
                case pickup:
                    std::println("counter");
                ;case pickup
                case post:
                    std::println("mail");
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
    std::println($price);
;MAIN
