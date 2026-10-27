# A module names the root's types through `my.` (owner, 2026-10-06: `my.` is the root), as std's `str.cy` names
# `my.casing`: in a parameter, a return type and a value built.
union.tagged Color:
    red;
    blue;
end of union.tagged Color
record P:
    x: integer.signed.32_bit;
end of record P

module paint:
    public function name [$c: my.Color] -> string.utf8:
        match $c:
            case red:
                return "red";
            end of case red
            case blue:
                return "blue";
            end of case blue
        end of match
    end of function name
    public function twice [$p: my.P] -> my.P:
        return my.P[x = $p.x times 2];
    end of function twice
end of module paint

!MAIN:
    let $q := paint.twice(P[x = 4]);
    std::print.shortest(paint.name(Color.blue), " ", $q.x, newline);
!end of MAIN
