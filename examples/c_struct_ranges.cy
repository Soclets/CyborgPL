# A range type may be a field of a `layout.C struct` (owner, 2026-09-27: a). C sees the
# plain number (Rect is two int32s, 8 bytes); the range is CyborgPL's promise — proven where
# CyborgPL fills one in, and checked at the door (`out_of_range`) when one comes from C.
%layout.C rec Rect:
    var w: int.sign.32[min: 0, max: 4096];
    var h: int.sign.32[min: 0, max: 4096];
end of rec Rect
%extern.C proc DrawRect [$r: Rect];
!MAIN:
    let $r := Rect[w = 640, h = 480];
    std::print.shortest($r.w, "x", $r.h, " ", size.of[Rect], newln);
!end of MAIN
