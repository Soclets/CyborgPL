# A `match` gives a value (owner, 2026-09-29: a): a case whose block ends
# in a value gives it, and a case may still leave; an `otherwise` arm gives
# the call's value the same way (examples/match_value.cpls as an exit code:
# free is 4 bytes, "taken by ann" 12, locked leaves with 8, and "12x" is
# not a number, -2 → 4 + 12 + 8 + 100 - 2 = 122).
extern.C fn exit ['code': int32];
enum Status { empty, claimed: str.utf8, locked }
fn label ['s': Status] -> str.utf8 {
    let 'text' := match 's' {
        empty         { is "free"; }
        claimed 'who' { is "taken by " concat 'who'; }
        locked        { return "(locked)"; }
    };
    return give 'text';
}
fn width ['s': Status] -> int32 {
    let 'l' := label[give 's'];
    return to.int32[str.bytes[lend 'l']];
}
MAIN {
    let 'n' := to.int32["12x"] otherwise { not_a_number { is -2; } out_of_range { is -1; } };
    let 'k': int32 = match Status.empty {
        empty { is 100; }
        claimed 'w' { is 0; }
        locked { is 0; }
    };
    no_check my.exit[width[Status.empty] plus width[Status.claimed["ann"]] plus width[Status.locked] plus 'k' plus 'n'];
}
