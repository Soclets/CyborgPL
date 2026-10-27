# a function carrying a copy/give capture is owned: copying it takes a word (owner, 2026-09-24: a)
!MAIN:
    let $xs := [1, 2];
    let $f := function [] -> integer.unsigned.64_bit with [copy $xs]:
        return std::list.length(lend $xs);
    end of function;
    let $g := $f;
!end of MAIN
