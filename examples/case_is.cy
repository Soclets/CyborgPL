# A case says its value with `is` (owner, 2026-10-07; `#strict`'s `giveS` before), and may do
# other work first.
union.tagged Status:
    empty; claimed: string.utf8; locked;
end of union.tagged Status
function label [$s: Status] -> string.utf8:
    let $text := match $s:
        case empty:
            is "free";
        end of case empty
        case claimed $who:
            let $prefix := "taken by "; is $prefix concatenate $who;
        end of case claimed
        case locked:
            is "locked";
        end of case locked
    end of match;
    return give $text;
end of function label
!MAIN:
    std::print.shortest(label(Status.claimed("bo")), newline);
!end of MAIN
