# A range type may be a field of a `layout.C struct` (owner, 2026-09-27: a). C sees the
# plain number (Rect is two int32s, 8 bytes); the range is CyborgPL's promise — proven where
# CyborgPL fills one in, and checked at the door (`out_of_range`) when one comes from C.
%layout.C rec Rect:
    var w: i32[min: 0, max: 4096];
    var h: i32[min: 0, max: 4096];
;%layout.C rec Rect
%extern.C proc DrawRect [let $r: Rect];
MAIN:
    let $r := Rect[w = 640, h = 480];
    std::print($r.w, "x", $r.h, " ", size.of[Rect], newln);
;MAIN
