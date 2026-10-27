# A `match` that gives a value (owner, 2026-09-29: a): a case whose block holds only a value is
# that value; a case may still leave instead (`return`, `fail`, `break`). The same holds for
# `otherwise` with several reasons. Every case is still covered.
union.tagged Status:
    empty;
    claimed: string.utf8;
    locked;
end of union.tagged Status
function label [$s: Status] -> string.utf8:
    let $text := match $s:
        case empty:
            is "free";
        end of case empty
        case claimed $who:
            is "taken by " concatenate $who;
        end of case claimed
        case locked:
            return "(locked)";
        end of case locked
    end of match;
    return give $text;
end of function label
!MAIN:
    let $n := to.integer.signed.32_bit("12x") otherwise:
        case not_a_number:
            is negative 2;
        end of case not_a_number
        case out_of_range:
            is negative 1;
        end of case out_of_range
    end of otherwise;
    std::print.shortest(label(Status.empty), " / ", label(Status.claimed("ann")), " / ", label(Status.locked), " / ", $n, newline);
    match Status.empty:
        case empty:
            std::print.shortest("statement match still works", newline);
        end of case empty
        case claimed $w:
        end of case claimed
        case locked:
        end of case locked
    end of match
!end of MAIN
