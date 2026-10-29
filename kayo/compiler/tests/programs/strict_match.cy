# In a `#strict` file a case says its value with `gives` (owner, 2026-09-29:
# b), and may do other work first (examples/strict_match.cpls as an exit
# code: "taken by bo" is 11 bytes).
%extern.C proc exit [$code: i32];
union.tag Status:
    empty; claimed: str.utf8; locked;
;union.tag Status
fn label [$s: Status] -> str.utf8:
    let $text := match $s:
        case empty:
            is "free";
        ;case empty
        case claimed $who:
            let $prefix := "taken by "; is $prefix concat $who;
        ;case claimed
        case locked:
            is "locked";
        ;case locked
    ;match;
    ret give $text;
;fn label
fn width [$s: Status] -> i32:
    let $l := label(give $s);
    let $n := std::str.bytes(lend $l);
    if $n > 1000:
        ret 0;
    ;if
    ret to.i32($n);
;fn width
MAIN:
    no_check my.exit(width(Status.claimed("bo")));
;MAIN
