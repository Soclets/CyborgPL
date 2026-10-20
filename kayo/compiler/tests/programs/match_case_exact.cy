# A `match` used as a value hands its type to each case (check claude,
# 2026-09-29, seed 1011): the case's bare numbers are exact, as in a
# declaration — (14576284062462565771 div 6) mod 51 is 25 — where they had
# been worked out at 64 bits and the asserted build printed 16.
extern.C fn exit ['code': i32];
enum E { a, b }
MAIN {
    let 'v': u64 = match E.a { a { is (14576284062462565771 div.floor 6) mod.floor 51; } b { is 1; } };
    let 'c': i32 = to.i32['v'];
    no_check my.exit['c'];
}
