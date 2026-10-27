# A collection says its function's labels and defaults (owner, 2026-10-21): `labels [$text as value]` gives a
# parameter its label, written `value:` in the call; `defaults [$times = 1]` what a left-out argument is. A `pub`
# function says whether its labels keep their order, `labels.in_order` or `labels.any_order`; a private one's plain
# `labels` keeps it. The arguments without a label come first, in place. Versions go in a `versions` block.
collection.function say:
    labels [$text as value, $times as count];
    defaults [$times = 1];
    say [$n: integer.signed.32_bit, $text: string.utf8, $times: integer.signed.32_bit] -> string.utf8:
        return to.stored.string.utf8($n) concatenate ": " concatenate $text concatenate " x" concatenate to.stored.string.utf8($times);
    end of say
end of collection.function say

public collection.function box:
    labels.any_order [$w as width, $h as height, $b as border];
    defaults [$b = 0];
    box [$w: integer.unsigned.32_bit, $h: integer.unsigned.32_bit, $b: integer.unsigned.32_bit] -> integer.unsigned.32_bit:
        return ($w plus $b plus $b) times ($h plus $b plus $b);
    end of box
end of collection.function box

collection.function area:
    versions area:
        [$side: integer.signed.64_bit] -> integer.signed.64_bit:
            return $side times $side;
        end of version
        [$w: integer.signed.64_bit, $h: integer.signed.64_bit] -> integer.signed.64_bit:
            return $w times $h;
        end of version
    end of versions area
end of collection.function area

!MAIN:
    std::print.shortest(say(2, value: "hi"), newline);
    std::print.shortest(say(3, value: "yo", count: 4), newline);
    std::print.shortest(box(height: 4, width: 10), " ", box(border: 1, width: 10, height: 4), newline);
    std::print.shortest(area(3), " ", area(3, 4), newline);
!end of MAIN
