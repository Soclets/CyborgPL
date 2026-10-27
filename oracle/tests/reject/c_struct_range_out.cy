# a C struct's ranged field is proven where Cyborg fills it in (2026-09-27)
%layout.C record Rect:
    w: integer.signed.32_bit[minimum: 0, maximum: 4096], h: integer.signed.32_bit[minimum: 0, maximum: 4096]
end of layout
!MAIN:
    let $r := Rect[w = 5000, h = 1];
!end of MAIN
