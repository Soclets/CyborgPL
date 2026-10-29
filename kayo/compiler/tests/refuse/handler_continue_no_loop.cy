# refuse: this handler can `break` or `continue`, so it is used only inside a loop
# A handler's arms are written for the caller's loop: one that can `continue`, used
# where no loop is, is refused at the use (owner, 2026-10-27: (b); oracle 27c9b24a).
coll.handler failed:
    member failed.skip:
        case bad:
            continue;
        ;case bad
    ;member failed.skip
;coll.handler failed
fn small [$n: i32] -> i32 or fail[bad]:
    if $n > 2:
        fail bad;
    ;if
    ret $n;
;fn small
MAIN:
    let $x := small(1) otherwise failed.*skip;
    std::print($x, newln);
;MAIN
