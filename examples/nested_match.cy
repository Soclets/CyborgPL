# A `match` alone in a case is a statement (owner, 2026-09-29); a nested match that is the case's value
# says `is match …` (owner, 2026-10-07: a case says its value with `is`).
union.tagged Order:
    cancelled;
    paid;
end of union.tagged Order
union.tagged Ship:
    pickup;
    post;
end of union.tagged Ship
!MAIN:
    let $o := Order.paid;
    let $s := Ship.post;
    match $o:
        case cancelled:
        end of case cancelled
        case paid:
            match $s:
                case pickup:
                    std::print.shortest("counter", newline);
                end of case pickup
                case post:
                    std::print.shortest("mail", newline);
                end of case post
            end of match
        end of case paid
    end of match
    let $price := match $o:
        case cancelled:
            is 0;
        end of case cancelled
        case paid:
            is match $s:
                case pickup:
                    is 5;
                end of case pickup
                case post:
                    is 9;
                end of case post
            end of match;
        end of case paid
    end of match;
    std::print.shortest($price, newline);
!end of MAIN
