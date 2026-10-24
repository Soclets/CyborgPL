# two meanings of one operator for the same types (2026-09-23)
struct A { x: i32 }
op <+> [$a: A, $b: A] -> i32 { return 1; }
op <+> [$a: A, $b: A] -> i32 { return 2; }
MAIN { }
