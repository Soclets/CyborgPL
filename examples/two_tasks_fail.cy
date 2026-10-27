# Two failing tasks in one block: each handler runs where its own `concur` was written
# (found by the compiler session, 2026-09-23). Exits with 1 plus 10 = 11.
%extern.C proc exit [$code: int.sign.32];
fn judge [$n: int.sign.32] -> fail[small]:
    if $n less_than 10:
        fail small;
    end of if
end of fn judge
!MAIN:
    var $seen: int.sign.32 = 0;
    block:
        %spawn.task:
            judge(3) otherwise fail;
        end of %spawn.task
        otherwise:
            case small:
                change $seen = $seen plus 1;
            end of case small
        end of otherwise;
        %spawn.task:
            judge(4) otherwise fail;
        end of %spawn.task
        otherwise:
            case small:
                change $seen = $seen plus 10;
            end of case small
        end of otherwise;
    end of block
    !no_check my.exit($seen);   # bare `exit` could be C's or std's (owner, 2026-10-06: a kit's own does not win)
!end of MAIN
