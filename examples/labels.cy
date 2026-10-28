# A collection says its function's labels and defaults (owner, 2026-10-21): `labels [$text as value]` gives a
# parameter its label, written `value:` in the call; `defaults [$times = 1]` what a left-out argument is. A `pub`
# function says whether its labels keep their order, `labels.in_order` or `labels.any_order`; a private one's plain
# `labels` keeps it. The arguments without a label come first, in place. Versions go in a `versions` block.
coll.fn say:
    labels [$text as value, $times as count];
    defaults [$times = 1];
    member say [let $n: i32, let $text: str.utf8, let $times: i32] -> str.utf8:
        ret to.stored.str.utf8($n) ++ ": " ++ $text ++ " x" ++ to.stored.str.utf8($times);
    ;member say
;coll.fn say

pub coll.fn box:
    labels.any_order [$w as width, $h as height, $b as border];
    defaults [$b = 0];
    member box [let $w: u32, let $h: u32, let $b: u32] -> u32:
        ret ($w + $b + $b) x ($h + $b + $b);
    ;member box
;coll.fn box

coll.fn area:
    versions area:
        [let $side: i64] -> i64:
            ret $side x $side;
        ;version
        [let $w: i64, let $h: i64] -> i64:
            ret $w x $h;
        ;version
    ;versions area
;coll.fn area

MAIN:
    std::println(say(2, value: "hi"));
    std::println(say(3, value: "yo", count: 4));
    std::println(box(height: 4, width: 10), " ", box(border: 1, width: 10, height: 4));
    std::println(area(3), " ", area(3, 4));
;MAIN
