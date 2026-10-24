# A `match` that gives a value (owner, 2026-09-29: a): a case whose block holds only a value is
# that value; a case may still leave instead (`return`, `fail`, `break`). The same holds for
# `otherwise` with several reasons. Every case is still covered.
enum Status { empty, claimed: str.utf8, locked }
fn label [$s: Status] -> str.utf8 {
    let $text := match $s {
        empty         { is "free"; }
        claimed $who { is "taken by " concat $who; }
        locked        { return "(locked)"; }
    };
    return give $text;
}
MAIN {
    let $n := to.i32("12x") otherwise { not_a_number { is -2; } out_of_range { is -1; } };
    std::print.useful(label(Status.empty), " / ", label(Status.claimed("ann")), " / ", label(Status.locked), " / ", $n, newln);
    match Status.empty {
        empty { std::print.useful("statement match still works", newln); }
        claimed $w { }
        locked { }
    }
}
