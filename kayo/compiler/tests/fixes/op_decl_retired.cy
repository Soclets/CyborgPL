# old_spelling: an operator declared with a retired symbol.
struct V { n: i32 }
op <= [$a: V, $b: V] -> bool { return $a.n < $b.n; }
MAIN { }
