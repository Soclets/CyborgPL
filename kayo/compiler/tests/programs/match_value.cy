# A `match` gives a value (owner, 2026-09-29: a): a case whose block ends
# in a value gives it, and a case may still leave; an `otherwise` arm gives
# the call's value the same way (examples/match_value.cpls as an exit code:
# free is 4 bytes, "taken by ann" 12, locked leaves with 8, and "12x" is
# not a number, -2 → 4 + 12 + 8 + 100 - 2 = 122).
%extern.C proc exit [let $code: i32];
union.tag Status:
    empty; claimed: str.utf8; locked;
;union.tag Status
fn label [let $s: Status] -> str.utf8:
    let $text := match $s:
        case empty:
            is "free";
        ;case empty
        case claimed $who:
            is "taken by " concat $who;
        ;case claimed
        case locked:
            ret "(locked)";
        ;case locked
    ;match;
    ret give $text;
;fn label
fn width [let $s: Status] -> i32:
    let $l := label(give $s);
    ret to.i32(std::str.bytes(lend $l));
;fn width
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
    let $k: i32 := match Status.empty:
        case empty:
            is 100;
        ;case empty
        case claimed $w:
            is 0;
        ;case claimed
        case locked:
            is 0;
        ;case locked
    ;match;
    no_check my.exit(width(Status.empty) + width(Status.claimed("ann")) + width(Status.locked) + $k + $n);
;MAIN
