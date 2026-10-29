# refuse: the value can be outside what a `u32` holds
# A `match` arm's answer lands in the place the match is stored into, checked
# like any store: -1 into a u32 went through unchecked (check claude,
# seed 102).
%extern.C proc exit [$code: i32];
union.tag E:
    c1; c2;
;union.tag E
fn zero [] -> u32:
    ret 0;
;fn zero
fn pick [] -> E:
    ret E.c2;
;fn pick
MAIN:
    let $w: u32 := zero();
    let $v: u32 := match pick():
        case c1:
            is 1;
        ;case c1
        case c2:
            is $w - 1;
        ;case c2
    ;match;
    no_check my.exit(to.i32($v mod.floor 100));
;MAIN
