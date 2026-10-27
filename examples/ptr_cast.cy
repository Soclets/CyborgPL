# A pointer seen as pointing at another type (owner, 2026-09-30: a): `ptr.cast[T]($p)` keeps the
# address and whether it may write, and changes only what it claims to point at. Always no_check.
%external.C procedure malloc [$bytes: integer.unsigned.64_bit] -> pointer.mutable[integer.unsigned.8_bit];
%external.C procedure free [$p: pointer.mutable[integer.unsigned.8_bit]];

%layout.C record Point:
    x: integer.signed.32_bit,
    y: integer.signed.32_bit
end of layout

!MAIN:
    !no_check let $raw := malloc(size.of[Point]);
    !no_check let $p := pointer.cast[Point]($raw);
    !no_check pointer.write($p, value: Point[x = 3, y = 4]);
    !no_check let $q := pointer.read($p);
    std::print.stored($q.x, " ", $q.y, newline);
    !no_check free($raw);
!end of MAIN
