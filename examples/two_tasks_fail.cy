# Two failing tasks in one block: each handler runs where its own `concur` was written
# (found by the compiler session, 2026-09-23). Exits with 1 plus 10 = 11.
%extern.C proc exit [let $code: i32];
fn judge [let $n: i32] -> fail[small]:
    if $n < 10:
        fail small;
    ;if
;fn judge
MAIN:
    var $seen: i32 = 0;
    block:
        %spawn.task:
            judge(3)?;
        ;%spawn.task
        otherwise:
            case small:
                change $seen to $seen + 1;
            ;case small
        ;otherwise;
        %spawn.task:
            judge(4)?;
        ;%spawn.task
        otherwise:
            case small:
                change $seen to $seen + 10;
            ;case small
        ;otherwise;
    ;block
    no_check my.exit($seen);   # bare `exit` could be C's or std's (owner, 2026-10-06: a kit's own does not win)
;MAIN
