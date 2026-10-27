# A collection says its function's labels and defaults (owner, 2026-10-21): `labels [$text as value]` gives a
# parameter its label, written `value:` in the call; `defaults [$times = 1]` what a left-out argument is. A `pub`
# function says whether its labels keep their order, `labels.in_order` or `labels.any_order`; a private one's plain
# `labels` keeps it. The arguments without a label come first, in place. Versions go in a `versions` block.
coll.fn say:
    labels [$text as value, $times as count];
    defaults [$times = 1];
    member say [let $n: int.sign.32, let $text: str.utf8, let $times: int.sign.32] -> str.utf8:
        ret to.stored.str.utf8($n) concat ": " concat $text concat " x" concat to.stored.str.utf8($times);
    end of member say
end of coll.fn say

pub coll.fn box:
    labels.any_order [$w as width, $h as height, $b as border];
    defaults [$b = 0];
    member box [let $w: int.unsign.32, let $h: int.unsign.32, let $b: int.unsign.32] -> int.unsign.32:
        ret ($w plus $b plus $b) times ($h plus $b plus $b);
    end of member box
end of coll.fn box

coll.fn area:
    versions area:
        [let $side: int.sign.64] -> int.sign.64:
            ret $side times $side;
        end of version
        [let $w: int.sign.64, let $h: int.sign.64] -> int.sign.64:
            ret $w times $h;
        end of version
    end of versions area
end of coll.fn area

!MAIN:
    std::print.shortest(say(2, value: "hi"), newln);
    std::print.shortest(say(3, value: "yo", count: 4), newln);
    std::print.shortest(box(height: 4, width: 10), " ", box(border: 1, width: 10, height: 4), newln);
    std::print.shortest(area(3), " ", area(3, 4), newln);
!end of MAIN
