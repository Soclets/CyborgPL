# A module names the root's types through `my.` (owner, 2026-10-06: `my.` is the root), as std's `str.cy` names
# `my.casing`: in a parameter, a return type and a value built.
union.tag Color:
    red;
    blue;
end of union.tag Color
rec P:
    var x: int.sign.32;
end of rec P

module paint:
    pub fn name [$c: my.Color] -> str.utf8:
        match $c:
            case red:
                ret "red";
            end of case red
            case blue:
                ret "blue";
            end of case blue
        end of match
    end of fn name
    pub fn twice [$p: my.P] -> my.P:
        ret my.P[x = $p.x times 2];
    end of fn twice
end of module paint

!MAIN:
    let $q := paint.twice(P[x = 4]);
    std::print.shortest(paint.name(Color.blue), " ", $q.x, newln);
!end of MAIN
