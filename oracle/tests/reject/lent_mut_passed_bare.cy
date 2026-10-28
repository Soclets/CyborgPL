proc add1 [lent.mut $out: std::ls[i32]]:
    std::ls.append(lend.mut $out, value: 1);
;proc add1
proc walk [lent.mut $out: std::ls[i32]]:
    add1($out); std::ls.append(lend.mut $out, value: 2);
;proc walk
MAIN:
    var $xs: std::ls[i32] = []; walk(lend.mut $xs); std::println(std::ls.len(lend $xs));
;MAIN
