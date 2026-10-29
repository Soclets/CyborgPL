# A module names the root's types through `my.` (owner, 2026-10-06: `my.` is the root), as std's `str.cy` names
# `my.casing`: in a parameter, a return type and a value built.
union.tag Color:
    red;
    blue;
;union.tag Color
rec P:
    var x: i32;
;rec P

module paint:
    pub fn name [$c: my.Color] -> str.utf8:
        match $c:
            case red:
                ret "red";
            ;case red
            case blue:
                ret "blue";
            ;case blue
        ;match
    ;fn name
    pub fn twice [$p: my.P] -> my.P:
        ret my.P[x = $p.x x 2];
    ;fn twice
;module paint

MAIN:
    let $q := paint.twice(P[x = 4]);
    std::print(paint.name(Color.blue), " ", $q.x, newln);
;MAIN
