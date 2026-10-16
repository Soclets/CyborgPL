# A range type may be a field of a `layout.C struct` (owner, 2026-09-27: a). C sees the
# plain number (Rect is two int32s, 8 bytes); the range is CyborgPL's promise — proven where
# CyborgPL fills one in, and checked at the door (`out_of_range`) when one comes from C.
layout.C struct Rect { w: int32[min 0, max 4096], h: int32[min 0, max 4096] }
extern.C fn DrawRect ['r': Rect];
MAIN {
    let 'r' := Rect[w = 640, h = 480];
    std::print.useful['r'.w, "x", 'r'.h, " ", size.of[Rect], newln];
}
