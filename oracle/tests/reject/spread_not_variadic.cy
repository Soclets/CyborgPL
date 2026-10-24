fn one [$a: i32] -> i32 { return $a; }
MAIN { let $xs := [1, 2]; let $t := one(...$xs); }
