# A `match` that gives a value (owner, 2026-09-29: a): a case whose block holds only a value is
# that value; a case may still leave instead (`return`, `fail`, `break`). The same holds for
# `otherwise` with several reasons. Every case is still covered.
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
            is "taken by " ++ $who;
        ;case claimed
        case locked:
            ret "(locked)";
        ;case locked
    ;match;
    ret give $text;
;fn label
MAIN:
    let $n := to.i32("12x")-
    otherwise:
        case not_a_number:
            is -2;
        ;case not_a_number
        case out_of_range:
            is -1;
        ;case out_of_range
    ;otherwise;
    std::print(label(Status.empty), " / ", label(Status.claimed("ann")), " / ", label(Status.locked), " / ", $n, newln);
    match Status.empty:
        case empty:
            std::print("statement match still works", newln);
        ;case empty
        case claimed $w:
        ;case claimed
        case locked:
        ;case locked
    ;match
;MAIN
