!MAIN:
    let $xs := [10, 20, 30];
    variable $n := std::list.length(lend $xs);   # u64
    change $n = $n minus 4;                          # -1 into a u64
!end of MAIN
