# A kit's front door (owner, 2026-10-19): a `coll.fn` whose members point at functions written elsewhere in the kit,
# each line writing the full name it makes. Users write geometry::area.square[…] whatever the folders look like.
pub coll.fn area:
    member area [$w: i64, $h: i64] -> i64:
        ret $w x $h;
    ;member area
    member area.square = shapes.square.area;
    member area.triangle = shapes.triangle.area;
    member area.unit [$w = 1, $h = 1];
;coll.fn area

MAIN:
    std::print(area(3, 4), " ", area.square(5), " ", area.triangle(6, 4), " ", area.unit(), newln);
;MAIN
