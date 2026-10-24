fn sum [...$nums: i32] -> i32 { return 0; }
fn f [lent $xs: std::list[i32]] -> i32 { return sum(...$xs); }
MAIN { let $a := [1, 2]; let $t := f(lend $a); }
