# A pointer seen as pointing at another type (owner, 2026-09-30: a): `ptr.cast[T]($p)` keeps the
# address and whether it may write, and changes only what it claims to point at. Always no_check.
%extern.C proc malloc [let $bytes: int.unsign.64] -> ptr.mut[int.unsign.8];
%extern.C proc free [let $p: ptr.mut[int.unsign.8]];

%layout.C rec Point:
    var x: int.sign.32;
    var y: int.sign.32;
end of rec Point

!MAIN:
    !no_check let $raw := malloc(size.of[Point]);
    !no_check let $p := ptr.cast[Point]($raw);
    !no_check ptr.write($p, value: Point[x = 3, y = 4]);
    !no_check let $q := ptr.read($p);
    std::print.stored($q.x, " ", $q.y, newln);
    !no_check free($raw);
!end of MAIN
