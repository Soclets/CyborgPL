# A pointer seen as pointing at another type (owner, 2026-09-30: a): `ptr.cast[T]($p)` keeps the
# address and whether it may write, and changes only what it claims to point at. Always no_check.
%extern.C proc malloc [$bytes: u64] -> ptr.mut[u8];
%extern.C proc free [$p: ptr.mut[u8]];

%layout.C rec Point:
    var x: i32;
    var y: i32;
;%layout.C rec Point

MAIN:
    no_check let $raw := malloc(size.of[Point]);
    no_check let $p := ptr.cast[Point]($raw);
    no_check ptr.write($p, value: Point[x = 3, y = 4]);
    no_check let $q := ptr.read($p);
    std::print.stored($q.x, " ", $q.y, newln);
    no_check free($raw);
;MAIN
