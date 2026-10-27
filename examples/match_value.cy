# A `match` that gives a value (owner, 2026-09-29: a): a case whose block holds only a value is
# that value; a case may still leave instead (`return`, `fail`, `break`). The same holds for
# `otherwise` with several reasons. Every case is still covered.
union.tag Status:
    empty;
    claimed: str.utf8;
    locked;
end of union.tag Status
fn label [let $s: Status] -> str.utf8:
    let $text := match $s:
        case empty:
            is "free";
        end of case empty
        case claimed $who:
            is "taken by " concat $who;
        end of case claimed
        case locked:
            ret "(locked)";
        end of case locked
    end of match;
    ret give $text;
end of fn label
!MAIN:
    let $n := to.int.sign.32("12x")-
    otherwise:
        case not_a_number:
            is neg 2;
        end of case not_a_number
        case out_of_range:
            is neg 1;
        end of case out_of_range
    end of otherwise;
    std::print.shortest(label(Status.empty), " / ", label(Status.claimed("ann")), " / ", label(Status.locked), " / ", $n, newln);
    match Status.empty:
        case empty:
            std::print.shortest("statement match still works", newln);
        end of case empty
        case claimed $w:
        end of case claimed
        case locked:
        end of case locked
    end of match
!end of MAIN
