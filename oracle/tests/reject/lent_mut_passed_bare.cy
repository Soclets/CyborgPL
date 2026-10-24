fn add1 [lent.mut $out: std::list[i32]] { std::list.append(lend.mut $out, value: 1); }
fn walk [lent.mut $out: std::list[i32]] { add1($out); std::list.append(lend.mut $out, value: 2); }
MAIN { let mut $xs: std::list[i32] = []; walk(lend.mut $xs); std::print.useful(std::list.length(lend $xs), newln); }
