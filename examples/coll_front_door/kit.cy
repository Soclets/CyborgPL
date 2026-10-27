# A kit's front door (owner, 2026-10-19): a `coll.fn` whose members point at functions written elsewhere in the kit,
# each line writing the full name it makes. Users write geometry::area.square[…] whatever the folders look like.
public collection.function area:
    area [$w: integer.signed.64_bit, $h: integer.signed.64_bit] -> integer.signed.64_bit:
        return $w times $h;
    end of area
    area.square = shapes.square.area;
    area.triangle = shapes.triangle.area;
    area.unit [$w = 1, $h = 1];
end of collection.function area

!MAIN:
    std::print.shortest(area(3, 4), " ", area.square(5), " ", area.triangle(6, 4), " ", area.unit(), newline);
!end of MAIN
