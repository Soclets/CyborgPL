# one narrowing becomes the other only by a conversion (owner, 2026-10-25)
!MAIN:
    let $a: std::ls.page_aligned[int.sign.32] := [5, 6];
    let $b: std::ls.not_page_aligned[int.sign.32] := copy $a;
!end of MAIN
