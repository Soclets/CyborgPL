# A function that captured with `copy` or `give` carries what it captured, so it is owned
# like a list: copying or handing it on takes a word (owner, 2026-09-24: a). A function
# carrying nothing, or only lends, copies by itself like a number.
!MAIN:
    let $xs := [1, 2, 3];
    let $count := fn [] -> int.unsign.64 with [copy $xs]:
        ret std::ls.len(lend $xs);
    end of fn;
    let $again := copy $count;        # its own copy of the list inside
    let $moved := give $count;        # 'count' is gone after this
    let $inc := fn [$a: int.sign.32] -> int.sign.32:
        ret $a plus 1;
    end of fn;
    let $inc2 := $inc;                # carries nothing: no word needed
    std::print.shortest($again(), " ", $moved(), " ", $inc2(41), newln);
!end of MAIN
