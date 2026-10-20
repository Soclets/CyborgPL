# A strict guard at a value inside the range (check claude, seed 7141):
# 'v' is any i32 C hands back, masked with `and.bit 16383`, which is 0
# to 16383 — `and.bit` with one side never negative never is — so
# `'v' == 0 or.else` leaves 1 to 16383, and `18 mod 'v'` cannot divide by
# 0. It was all of i32, 0 inside it, and the `mod` was refused.
# Whatever the pid, the guard holds (18 mod v is at most 18): 7.
extern.C fn exit ['code': i32];
extern.C fn getpid [] -> i32;
MAIN {
    no_check let immut 'p': i32 = getpid[];
    let immut 'v': i32 = 'p' and.bit 16383;
    let mut 'code': i32 = 0;
    if 'v' equal_to 0 or.else (18 mod.floor 'v') < 20 {
        change 'code' = 7;
    }
    no_check my.exit['code'];
}
