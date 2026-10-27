# A kit's front door (owner, 2026-10-19): a `coll.fn` whose members point at functions written elsewhere in the kit,
# each line writing the full name it makes. Users write geometry::area.square[…] whatever the folders look like.
pub coll.fn area:
    member area [let $w: int.sign.64, let $h: int.sign.64] -> int.sign.64:
        ret $w times $h;
    end of member area
    member area.square = shapes.square.area;
    member area.triangle = shapes.triangle.area;
    member area.unit [$w = 1, $h = 1];
end of coll.fn area

!MAIN:
    std::print.shortest(area(3, 4), " ", area.square(5), " ", area.triangle(6, 4), " ", area.unit(), newln);
!end of MAIN
