# a C struct's ranged field is proven where Cyborg fills it in (2026-09-27)
%layout.C rec Rect:
    var w: int.sign.32[min: 0, max: 4096];
    var h: int.sign.32[min: 0, max: 4096];
end of rec Rect
!MAIN:
    let $r := Rect[w = 5000, h = 1];
!end of MAIN
