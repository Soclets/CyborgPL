# `gives` is spelled `giveS`, the capital S keeping it apart from `give` (owner, 2026-09-29)
enum S { a, b }
MAIN { let immut 'x': int32 = match S.a { a { gives 1; } b { gives 2; } }; }
