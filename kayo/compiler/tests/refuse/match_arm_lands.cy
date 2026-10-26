# refuse: the value can be outside what a `u32` holds
# A `match` arm's answer lands in the place the match is stored into, checked
# like any store: -1 into a u32 went through unchecked (check claude,
# seed 102).
%extern.C fn exit [$code: i32];
enum E { c1, c2 }
fn zero [] -> u32 { return 0; }
fn pick [] -> E { return E.c2; }
MAIN {
    let $w: u32 = zero();
    let $v: u32 = match pick() {
        c1 { is 1; }
        c2 { is $w minus 1; }
    };
    no_check my.exit(to.i32($v mod.floor 100));
}
