# refuse: error[wrong_type]
# One narrowing is not the other (design claude, 2026-10-25): a page-aligned list given into a
# `std::list.not_page_aligned` place is refused.
MAIN:
    let $a: std::ls.page_aligned[i32] := [1, 2];
    let $b: std::ls.not_page_aligned[i32] := give $a;
;MAIN
