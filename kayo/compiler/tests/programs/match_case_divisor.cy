# Layer 3 must see every case of a match (check claude, 2026-09-29, seed
# 1990): 'd' shr 35 is 0 for 284, so 5 div 'v' divides by zero. The query
# kept only the first case's value, 1, and "proved" the division; and it
# shifted a u64 arithmetically. Now it traps, as the oracle stops.
extern.C fn exit [$code: i32];
enum E { a, b: u64 }
MAIN {
    let $v: u64 = match E.b(284) { a { is 1; } b $d { is $d shr 35; } };
    let $n: u64 = 5 div.floor $v;
    no_check my.exit(to.i32($n mod.floor 100));
}
