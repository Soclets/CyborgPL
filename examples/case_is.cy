# A case says its value with `is` (owner, 2026-10-07; `#strict`'s `giveS` before), and may do
# other work first.
union.tag Status:
    empty;
    claimed: str.utf8;
    locked;
;union.tag Status
fn label [let $s: Status] -> str.utf8:
    let $text := match $s:
        case empty:
            is "free";
        ;case empty
        case claimed $who:
            let $prefix := "taken by "; is $prefix ++ $who;
        ;case claimed
        case locked:
            is "locked";
        ;case locked
    ;match;
    ret give $text;
;fn label
MAIN:
    std::print(label(Status.claimed("bo")), newln);
;MAIN
