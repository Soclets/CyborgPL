!MAIN:
    let $xs := [10, 20, 30];
    var $n = std::ls.len(lend $xs);   # u64
    change $n to $n minus 4;                          # -1 into a u64
!end of MAIN
