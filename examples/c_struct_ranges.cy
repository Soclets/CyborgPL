# A range type may be a field of a `layout.C struct` (owner, 2026-09-27: a). C sees the
# plain number (Rect is two int32s, 8 bytes); the range is CyborgPL's promise — proven where
# CyborgPL fills one in, and checked at the door (`out_of_range`) when one comes from C.
%layout.C record Rect:
    w: integer.signed.32_bit[minimum: 0, maximum: 4096];
    h: integer.signed.32_bit[minimum: 0, maximum: 4096];
end of record Rect
%external.C procedure DrawRect [$r: Rect];
!MAIN:
    let $r := Rect[w = 640, h = 480];
    std::print.shortest($r.w, "x", $r.h, " ", size.of[Rect], newline);
!end of MAIN
