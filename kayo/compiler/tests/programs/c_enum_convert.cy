# A number from C becomes a C enum only through a check (owner, 2026-09-27):
# `to.Key[…]` fails `out_of_range` for a number no case has — 302, a key a
# newer library added — so the program decides what it means
# (examples/c_enum_convert.cpls as an exit code: 2, 0, 3 → 203).
%extern.C fn exit [$code: i32];
%layout.C enum Key { a = 65, b = 66, escape = 256 }
fn on_key [$code: i32] -> i32 {
    let $k := to.Key($code) otherwise { out_of_range { return 0; } };
    match $k {
        a { return 1; }
        b { return 2; }
        escape { return 3; }
    }
}
MAIN {
    no_check my.exit(on_key(66) times 100 plus on_key(302) times 10 plus on_key(256));
}
