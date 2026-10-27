# A function that captured with `copy` or `give` carries what it captured, so it is owned
# like a list: copying or handing it on takes a word (owner, 2026-09-24: a). A function
# carrying nothing, or only lends, copies by itself like a number.
!MAIN:
    let $xs := [1, 2, 3];
    let $count := function [] -> integer.unsigned.64_bit with [copy $xs]:
        return std::list.length(lend $xs);
    end of function;
    let $again := copy $count;        # its own copy of the list inside
    let $moved := give $count;        # 'count' is gone after this
    let $inc := function [$a: integer.signed.32_bit] -> integer.signed.32_bit:
        return $a plus 1;
    end of function;
    let $inc2 := $inc;                # carries nothing: no word needed
    std::print.shortest($again(), " ", $moved(), " ", $inc2(41), newline);
!end of MAIN
