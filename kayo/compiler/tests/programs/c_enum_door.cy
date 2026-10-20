# A C enum and a C struct with a ranged field cross to real C (owner,
# 2026-09-27: a): the enum is C's int, `div_t` comes back packed in one
# register as C returns it, and what C hands back is checked at the door —
# toupper['2'] is no case, 50 is past quot's range, and both take `otherwise`.
# The oracle cannot link C, so the answer is worked out by hand:
# 1000 + 200 + 30 + (3 + 1) + 9 = 1243, which exits as 1243 mod 256 = 219.
extern.C fn exit ['code': i32];
layout.C enum Key { a = 65, b = 66, escape = 256 }
extern.C fn abs ['k': Key] -> i32;
extern.C fn toupper ['c': i32] -> Key;
layout.C struct Div { quot: i32[min 0, max 10], rem: i32 }
extern.C fn div ['n': i32, 'd': i32] -> Div;

fn code ['k': Key] -> i32 {
    match 'k' {
        a { return 1; }
        b { return 2; }
        escape { return 3; }
    }
}

MAIN {
    no_check let 'n': i32 = abs[Key.escape];                        # 256
    no_check let 'up': Key = toupper[97] otherwise Key.escape;        # a
    no_check let 'same': Key = toupper[66] otherwise Key.escape;      # b
    no_check let 'none': Key = toupper[50] otherwise Key.escape;      # '2' is no case: escape
    no_check let 'q': Div = div[7, 2] otherwise Div[quot = 0, rem = 9];
    no_check let 'big': Div = div[100, 2] otherwise Div[quot = 0, rem = 9];
    no_check my.exit[('n' minus 256) plus code[give 'up'] times 1000 plus code[give 'same'] times 100 plus code[give 'none'] times 10 plus 'q'.quot plus 'q'.rem plus 'big'.rem];
}
