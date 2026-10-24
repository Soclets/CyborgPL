# `ptr.cast[T]['p']` sees the same address as pointing at a T (owner,
# 2026-09-30: a): malloc's bytes as a `layout.C` Point, written and read
# whole, then as int32s one at a time. 3 x 10 + 4 = 34, and the second
# i32 is y, 4: 34 + 4 = 38.
extern.C fn malloc [$bytes: u64] -> ptr.mut[u8];
extern.C fn free [$p: ptr.mut[u8]];
extern.C fn exit [$code: i32];

layout.C struct Point {
    x: i32,
    y: i32
}

MAIN {
    no_check let $raw := malloc(size.of[Point]);
    no_check let $p := ptr.cast[Point]($raw);
    no_check ptr.write($p, value: Point[x = 3, y = 4]);
    no_check let $q := ptr.read($p);
    no_check let $second := ptr.read(ptr.at(ptr.cast[i32]($raw), by: 1));
    let $code := $q.x times 10 plus $q.y plus $second;
    no_check free($raw);
    no_check my.exit($code);
}
