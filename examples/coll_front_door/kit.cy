# A kit's front door (owner, 2026-10-19): a `coll.fn` whose members point at functions written elsewhere in the kit,
# each line writing the full name it makes. Users write geometry::area.square[…] whatever the folders look like.
pub bare.never coll.fn area {
    area ['w': int64, 'h': int64] -> int64 { return 'w' x 'h'; }
    area.square = shapes.square.area;
    area.triangle = shapes.triangle.area;
    area.unit ['w' = 1, 'h' = 1];
}

MAIN {
    std::print.useful[area[3, 4], " ", area.square[5], " ", area.triangle[6, 4], " ", area.unit[], newln];
}
