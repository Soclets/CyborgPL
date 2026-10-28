# a C struct's ranged field is proven where Cyborg fills it in (2026-09-27)
%layout.C rec Rect:
    var w: i32[min: 0, max: 4096];
    var h: i32[min: 0, max: 4096];
;%layout.C rec Rect
MAIN:
    let $r := Rect[w = 5000, h = 1];
;MAIN
