# A case says its value with `is` (owner, 2026-10-07; `#strict`'s `giveS` before), and may do
# other work first.
union.tag Status:
    empty;
    claimed: str.utf8;
    locked;
end of union.tag Status
fn label [$s: Status] -> str.utf8:
    let $text := match $s:
        case empty:
            is "free";
        end of case empty
        case claimed $who:
            let $prefix := "taken by "; is $prefix concat $who;
        end of case claimed
        case locked:
            is "locked";
        end of case locked
    end of match;
    ret give $text;
end of fn label
!MAIN:
    std::print.shortest(label(Status.claimed("bo")), newln);
!end of MAIN
