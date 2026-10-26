# C's bit-fields laid out as clang lays them out on macOS (owner,
# 2026-10-19), seen through a union: clang puts Status's mode 5, ready 1 and
# level -8 in one byte, 141, and Mixed in three words, 5054277, 999999 and
# 30000 (each checked against a C program). 141 + 5054277 + 999999 + 30000
# is 6084417, and 6084417 mod 256 is 65.
%extern.C fn exit [$code: i32];
%layout.C struct Status { mode: u8[bits: 3], ready: bool[bits: 1], level: i8[bits: 4] }
%layout.C union SB { s: Status, b: u8 }
%layout.C struct Mixed { a: u8[bits: 3], b: u16[bits: 10], c: u8, d: u32[bits: 20], e: u32[bits: 15] }
%layout.C struct W { w0: u32, w1: u32, w2: u32 }
%layout.C union MB { m: Mixed, w: W }
MAIN {
    let $u := SB[s = Status[mode = 5, ready = true, level = -8]];
    no_check let $raw := $u.b;
    let $mb := MB[m = Mixed[a = 5, b = 1000, c = 77, d = 999999, e = 30000]];
    no_check let $w := copy $mb.w;
    let $t: u64 = to.u64($raw) + to.u64($w.w0) + to.u64($w.w1) + to.u64($w.w2);
    no_check my.exit(to.i32($t mod.floor 256));
}
