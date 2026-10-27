coll.handler failed:
    member failed.skip:
        !case bad:
            continue;
        !end of case bad
    end of member failed.skip
end of coll.handler failed

fn small [let $n: int.sign.32] -> int.sign.32 or fail[bad]:
    if $n gt 2:
        fail bad;
    end of if
    ret $n;
end of fn small

!MAIN:
    let $x := small(1) otherwise failed.!skip;
    std::print.shortest($x, newln);
!end of MAIN
